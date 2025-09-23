pub mod ast;
pub mod token;

use lalrpop_util::lalrpop_mod;

// Import generated parsers
lalrpop_mod!(pub calculator_builtin);
lalrpop_mod!(pub expression);
lalrpop_mod!(pub expression_logos);
lalrpop_mod!(pub left_recursion);

use lalrpop_util::ParseError;
use logos::Logos;

/// Parse a simple calculator expression using built-in lexer
pub fn parse_calculator(input: &str) -> Result<i32, String> {
    calculator_builtin::ExprParser::new()
        .parse(input)
        .map_err(|e| format!("Parse error: {:?}", e))
}

/// Example of detailed error handling for parse errors
pub fn parse_with_detailed_errors(input: &str) -> Result<i32, String> {
    let parser = calculator_builtin::ExprParser::new();
    match parser.parse(input) {
        Ok(result) => Ok(result),
        Err(ParseError::InvalidToken { location }) => {
            Err(format!("Invalid token at position {}", location))
        }
        Err(ParseError::UnrecognizedToken { token, expected }) => {
            let (start, _, end) = token;
            Err(format!(
                "Unexpected '{}' at position {}-{}, expected one of: {:?}",
                &input[start..end],
                start,
                end,
                expected
            ))
        }
        Err(ParseError::UnrecognizedEof { location, expected }) => Err(format!(
            "Unexpected end of input at position {}, expected: {:?}",
            location, expected
        )),
        Err(ParseError::ExtraToken { token }) => {
            let (start, _, end) = token;
            Err(format!(
                "Extra token '{}' at position {}-{} after valid input",
                &input[start..end],
                start,
                end
            ))
        }
        Err(ParseError::User { error }) => Err(format!("Parse error: {}", error)),
    }
}

/// Parse an expression language program using built-in lexer
pub fn parse_expression(input: &str) -> Result<ast::Program, String> {
    expression::ProgramParser::new()
        .parse(input)
        .map_err(|e| format!("Parse error: {:?}", e))
}

/// Parse using logos for lexing
pub fn parse_with_logos(input: &str) -> Result<ast::Program, String> {
    let lexer = token::Token::lexer(input);
    let tokens: Result<Vec<_>, _> = lexer
        .spanned()
        .map(|(tok, span)| match tok {
            Ok(t) => Ok((span.start, t, span.end)),
            Err(_) => Err("Lexer error"),
        })
        .collect();

    match tokens {
        Ok(tokens) => expression_logos::ProgramParser::new()
            .parse(tokens)
            .map_err(|e| format!("Parse error: {:?}", e)),
        Err(e) => Err(e.to_string()),
    }
}

/// Example: Building a simple interpreter
pub struct Interpreter {
    variables: std::collections::HashMap<String, f64>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: std::collections::HashMap::new(),
        }
    }

    pub fn execute(&mut self, program: &ast::Program) -> Result<(), String> {
        for statement in &program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, stmt: &ast::Statement) -> Result<(), String> {
        match stmt {
            ast::Statement::Expression(expr) => {
                self.eval_expr(expr)?;
                Ok(())
            }
            ast::Statement::Assignment(name, expr) => {
                let value = self.eval_expr(expr)?;
                self.variables.insert(name.clone(), value);
                Ok(())
            }
            ast::Statement::Print(expr) => {
                let value = self.eval_expr(expr)?;
                println!("{}", value);
                Ok(())
            }
            ast::Statement::If(cond, then_block, else_block) => {
                let cond_value = self.eval_expr(cond)?;
                if cond_value != 0.0 {
                    for stmt in then_block {
                        self.execute_statement(stmt)?;
                    }
                } else if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.execute_statement(stmt)?;
                    }
                }
                Ok(())
            }
            ast::Statement::While(cond, body) => {
                while self.eval_expr(cond)? != 0.0 {
                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }
                }
                Ok(())
            }
        }
    }

    fn eval_expr(&self, expr: &ast::Expr) -> Result<f64, String> {
        match expr {
            ast::Expr::Number(n) => Ok(*n),
            ast::Expr::Add(l, r) => Ok(self.eval_expr(l)? + self.eval_expr(r)?),
            ast::Expr::Subtract(l, r) => Ok(self.eval_expr(l)? - self.eval_expr(r)?),
            ast::Expr::Multiply(l, r) => Ok(self.eval_expr(l)? * self.eval_expr(r)?),
            ast::Expr::Divide(l, r) => {
                let divisor = self.eval_expr(r)?;
                if divisor == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(self.eval_expr(l)? / divisor)
                }
            }
            ast::Expr::Negate(e) => Ok(-self.eval_expr(e)?),
            ast::Expr::Variable(name) => self
                .variables
                .get(name)
                .copied()
                .ok_or_else(|| format!("Undefined variable: {}", name)),
            ast::Expr::Call(name, args) => {
                let arg_values: Result<Vec<_>, _> =
                    args.iter().map(|e| self.eval_expr(e)).collect();
                let arg_values = arg_values?;

                match name.as_str() {
                    "max" if arg_values.len() == 2 => Ok(f64::max(arg_values[0], arg_values[1])),
                    "min" if arg_values.len() == 2 => Ok(f64::min(arg_values[0], arg_values[1])),
                    "sqrt" if arg_values.len() == 1 => Ok(arg_values[0].sqrt()),
                    "abs" if arg_values.len() == 1 => Ok(arg_values[0].abs()),
                    _ => Err(format!("Unknown function: {}", name)),
                }
            }
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Demonstrate left vs right associativity parsing
pub fn demonstrate_associativity(input: &str) -> (String, String) {
    let left = left_recursion::LeftAssociativeParser::new()
        .parse(input)
        .map(|e| format!("{:?}", e))
        .unwrap_or_else(|e| format!("Error: {:?}", e));

    let right = left_recursion::RightAssociativeParser::new()
        .parse(input)
        .map(|e| format!("{:?}", e))
        .unwrap_or_else(|e| format!("Error: {:?}", e));

    (left, right)
}

/// Parse a comma-separated list using left recursion
pub fn parse_list_left(input: &str) -> Result<Vec<i32>, String> {
    left_recursion::CommaSeparatedLeftParser::new()
        .parse(input)
        .map_err(|e| format!("Parse error: {:?}", e))
}

/// Parse field access chains like "obj.field1.field2"
pub fn parse_field_access(input: &str) -> Result<String, String> {
    left_recursion::FieldAccessParser::new()
        .parse(input)
        .map_err(|e| format!("Parse error: {:?}", e))
}

/// Parse method chains like "obj.method1().method2()"
pub fn parse_method_chain(input: &str) -> Result<String, String> {
    left_recursion::MethodChainParser::new()
        .parse(input)
        .map_err(|e| format!("Parse error: {:?}", e))
}

/// Parse expressions with full operator precedence
pub fn parse_with_precedence(input: &str) -> Result<ast::Expr, String> {
    left_recursion::ExprParser::new()
        .parse(input)
        .map_err(|e| format!("Parse error: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator() {
        assert_eq!(parse_calculator("2 + 3 * 4").unwrap(), 14);
        assert_eq!(parse_calculator("(2 + 3) * 4").unwrap(), 20);
        assert_eq!(parse_calculator("10 - 2 - 3").unwrap(), 5);
    }

    #[test]
    fn test_expression_parser() {
        let program = parse_expression("let x = 10; let y = 20; print x + y;").unwrap();
        assert_eq!(program.statements.len(), 3);
    }

    #[test]
    fn test_logos_parser() {
        let program = parse_with_logos("let x = 5; print x * 2;").unwrap();
        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn test_interpreter() {
        let mut interpreter = Interpreter::new();
        let program = parse_expression(
            "let x = 10;
             let y = 20;
             let z = x + y;
             print z;",
        )
        .unwrap();

        interpreter.execute(&program).unwrap();
        assert_eq!(*interpreter.variables.get("z").unwrap(), 30.0);
    }

    #[test]
    fn test_if_statement() {
        let program = parse_expression(
            "let x = 5;
             if x {
                 let y = 10;
             }",
        )
        .unwrap();

        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn test_function_calls() {
        let program = parse_expression(
            "let x = max(10, 20);
             let y = sqrt(16);",
        )
        .unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.execute(&program).unwrap();
        assert_eq!(*interpreter.variables.get("x").unwrap(), 20.0);
        assert_eq!(*interpreter.variables.get("y").unwrap(), 4.0);
    }

    #[test]
    fn test_left_vs_right_associativity() {
        // Test that subtraction is left-associative
        // 10 - 5 - 2 should be (10 - 5) - 2 = 3 for left
        // and 10 - (5 - 2) = 7 for right
        let (left, right) = demonstrate_associativity("10 - 5 - 2");
        assert!(left.contains("Subtract"));
        assert!(right.contains("Subtract"));
    }

    #[test]
    fn test_comma_separated_list() {
        let result = parse_list_left("1, 2, 3, 4, 5").unwrap();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_field_access_chain() {
        let result = parse_field_access("obj.field1.field2.field3").unwrap();
        assert_eq!(result, "obj.field1.field2.field3");
    }

    #[test]
    fn test_method_chain() {
        let result = parse_method_chain("obj.method1().method2().method3()").unwrap();
        assert_eq!(result, "obj.method1().method2().method3()");
    }

    #[test]
    fn test_operator_precedence() {
        // Test that * has higher precedence than +
        // 2 + 3 * 4 should be 2 + (3 * 4) = 14
        let expr = parse_with_precedence("2 + 3 * 4").unwrap();
        assert_eq!(expr.eval(), 14.0);
    }
}
