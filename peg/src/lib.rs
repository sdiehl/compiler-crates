use std::collections::HashMap;

/// AST nodes for a functional programming language
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
    Let {
        bindings: Vec<(String, Expr)>,
        body: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    List(Vec<Expr>),
    Record(HashMap<String, Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Cons,
    Append,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expr),
    Definition {
        name: String,
        value: Expr,
    },
    TypeDef {
        name: String,
        constructors: Vec<(String, Vec<String>)>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// Error type for parser errors
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub expected: Vec<String>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

peg::parser! {
    pub grammar functional_parser() for str {
        /// Parse a complete program
        pub rule program() -> Program
            = _ statements:statement()* _ {
                Program { statements }
            }

        /// Parse a statement
        rule statement() -> Statement
            = definition() / type_definition() / expression_statement()

        /// Parse a variable definition
        rule definition() -> Statement
            = "def" _ name:identifier() _ "=" _ value:expression() _ {
                Statement::Definition { name, value }
            }

        /// Parse a type definition
        rule type_definition() -> Statement
            = "type" _ name:identifier() _ "=" _ constructors:constructor_list() _ {
                Statement::TypeDef { name, constructors }
            }

        /// Parse constructor list for type definitions
        rule constructor_list() -> Vec<(String, Vec<String>)>
            = head:constructor() tail:(_ "|" _ c:constructor() { c })* {
                let mut result = vec![head];
                result.extend(tail);
                result
            }

        /// Parse a constructor
        rule constructor() -> (String, Vec<String>)
            = name:identifier() args:(_ "(" _ args:type_list() _ ")" { args })? {
                (name, args.unwrap_or_default())
            }

        /// Parse a list of types
        rule type_list() -> Vec<String>
            = head:identifier() tail:(_ "," _ t:identifier() { t })* {
                let mut result = vec![head];
                result.extend(tail);
                result
            }

        /// Parse an expression statement
        rule expression_statement() -> Statement
            = expr:expression() {
                Statement::Expression(expr)
            }

        /// Parse expressions with left-associative operators
        pub rule expression() -> Expr = precedence!{
            x:(@) _ "||" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::Or, right: Box::new(y) } }
            --
            x:(@) _ "&&" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::And, right: Box::new(y) } }
            --
            x:(@) _ "==" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::Eq, right: Box::new(y) } }
            x:(@) _ "!=" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::Ne, right: Box::new(y) } }
            --
            x:(@) _ "<=" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::Le, right: Box::new(y) } }
            x:(@) _ ">=" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::Ge, right: Box::new(y) } }
            x:(@) _ "<" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::Lt, right: Box::new(y) } }
            x:(@) _ ">" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::Gt, right: Box::new(y) } }
            --
            x:(@) _ "+" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::Add, right: Box::new(y) } }
            x:(@) _ "-" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::Sub, right: Box::new(y) } }
            --
            x:(@) _ "*" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::Mul, right: Box::new(y) } }
            x:(@) _ "/" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::Div, right: Box::new(y) } }
            x:(@) _ "%" _ y:@ { Expr::Binary { left: Box::new(x), op: BinaryOp::Mod, right: Box::new(y) } }
            --
            x:@ _ "**" _ y:(@) { Expr::Binary { left: Box::new(x), op: BinaryOp::Pow, right: Box::new(y) } }
            --
            "-" _ e:@ { Expr::Unary { op: UnaryOp::Neg, expr: Box::new(e) } }
            "not" _ e:@ { Expr::Unary { op: UnaryOp::Not, expr: Box::new(e) } }
            --
            e:postfix() { e }
        }

        /// Postfix expressions (function calls)
        rule postfix() -> Expr
            = e:atom() calls:call_suffix()* {
                calls.into_iter().fold(e, |func, args| {
                    Expr::Call { func: Box::new(func), args }
                })
            }

        rule call_suffix() -> Vec<Expr>
            = _ "(" _ args:argument_list() _ ")" { args }

        /// Parse atomic expressions
        rule atom() -> Expr
            = float()  // Must come before number
            / number()
            / string_literal()
            / boolean()
            / list()
            / record()
            / lambda()
            / let_expression()
            / if_expression()
            / identifier_expr()
            / "(" _ e:expression() _ ")" { e }

        /// Parse numbers (integers only)
        rule number() -> Expr
            = n:$("-"? ['0'..='9']+) !("." ['0'..='9']) {?
                n.parse::<i64>()
                    .map(Expr::Number)
                    .map_err(|_| "number")
            }

        /// Parse floating-point numbers
        rule float() -> Expr
            = n:$("-"? ['0'..='9']+ "." ['0'..='9']+) {?
                n.parse::<f64>()
                    .map(Expr::Float)
                    .map_err(|_| "float")
            }

        /// Parse string literals
        rule string_literal() -> Expr
            = "\"" chars:string_char()* "\"" {
                Expr::String(chars.into_iter().collect())
            }

        /// Parse string characters with escape sequences
        rule string_char() -> char
            = "\\\\" { '\\' }
            / "\\\"" { '"' }
            / "\\n" { '\n' }
            / "\\t" { '\t' }
            / "\\r" { '\r' }
            / !['"' | '\\'] c:char() { c }

        /// Parse any character
        rule char() -> char
            = c:$([_]) { c.chars().next().unwrap() }

        /// Parse boolean literals
        rule boolean() -> Expr
            = "true" !identifier_char() { Expr::Bool(true) }
            / "false" !identifier_char() { Expr::Bool(false) }

        /// Parse lists
        rule list() -> Expr
            = "[" _ elements:expression_list() _ "]" {
                Expr::List(elements)
            }

        /// Parse expression lists
        rule expression_list() -> Vec<Expr>
            = head:expression() tail:(_ "," _ e:expression() { e })* {
                let mut result = vec![head];
                result.extend(tail);
                result
            } / { vec![] }

        /// Parse argument lists (for function calls)
        rule argument_list() -> Vec<Expr>
            = expression_list()

        /// Parse records (key-value mappings)
        rule record() -> Expr
            = "{" _ fields:field_list() _ "}" {
                Expr::Record(fields.into_iter().collect())
            }

        /// Parse field lists for records
        rule field_list() -> Vec<(String, Expr)>
            = head:field() tail:(_ "," _ f:field() { f })* {
                let mut result = vec![head];
                result.extend(tail);
                result
            } / { vec![] }

        /// Parse a single field
        rule field() -> (String, Expr)
            = key:identifier() _ ":" _ value:expression() {
                (key, value)
            }

        /// Parse lambda expressions
        rule lambda() -> Expr
            = "\\" _ params:parameter_list() _ "->" _ body:expression() {
                Expr::Lambda { params, body: Box::new(body) }
            }
            / "fn" _ params:parameter_list() _ "->" _ body:expression() {
                Expr::Lambda { params, body: Box::new(body) }
            }

        /// Parse parameter lists
        rule parameter_list() -> Vec<String>
            = "(" _ params:identifier_list() _ ")" { params }
            / param:identifier() { vec![param] }

        /// Parse identifier lists
        rule identifier_list() -> Vec<String>
            = head:identifier() tail:(_ "," _ id:identifier() { id })* {
                let mut result = vec![head];
                result.extend(tail);
                result
            } / { vec![] }

        /// Parse let expressions
        rule let_expression() -> Expr
            = "let" _ bindings:binding_list() _ "in" _ body:expression() {
                Expr::Let { bindings, body: Box::new(body) }
            }

        /// Parse binding lists for let expressions
        rule binding_list() -> Vec<(String, Expr)>
            = head:binding() tail:(_ "," _ b:binding() { b })* {
                let mut result = vec![head];
                result.extend(tail);
                result
            }

        /// Parse a single binding
        rule binding() -> (String, Expr)
            = name:identifier() _ "=" _ value:expression() {
                (name, value)
            }

        /// Parse if expressions
        rule if_expression() -> Expr
            = "if" _ cond:expression() _ "then" _ then_branch:expression()
              else_branch:(_ "else" _ e:expression() { e })? {
                Expr::If {
                    condition: Box::new(cond),
                    then_branch: Box::new(then_branch),
                    else_branch: else_branch.map(Box::new),
                }
            }

        /// Parse identifier expressions
        rule identifier_expr() -> Expr
            = id:identifier() { Expr::Identifier(id) }

        /// Parse identifiers
        rule identifier() -> String
            = !reserved_word() s:$(identifier_start() identifier_char()*) { s.to_string() }

        rule identifier_start() -> ()
            = ['a'..='z' | 'A'..='Z' | '_'] {}

        rule identifier_char() -> ()
            = ['a'..='z' | 'A'..='Z' | '0'..='9' | '_'] {}

        /// Reserved words that can't be identifiers
        rule reserved_word()
            = ("if" / "then" / "else" / "let" / "in" / "fn" / "def" / "type"
               / "true" / "false" / "not") !identifier_char()

        /// Whitespace
        rule _() = quiet!{ (whitespace() / comment())* }

        rule whitespace()
            = [' ' | '\t' | '\n' | '\r']+

        rule comment()
            = "//" (!"\n" [_])*
            / "/*" (!"*/" [_])* "*/"
    }
}

/// Simple evaluator for mathematical expressions
pub fn evaluate(expr: &Expr) -> Result<f64, String> {
    match expr {
        Expr::Number(n) => Ok(*n as f64),
        Expr::Float(f) => Ok(*f),
        Expr::Binary { left, op, right } => {
            let l = evaluate(left)?;
            let r = evaluate(right)?;
            match op {
                BinaryOp::Add => Ok(l + r),
                BinaryOp::Sub => Ok(l - r),
                BinaryOp::Mul => Ok(l * r),
                BinaryOp::Div => {
                    if r == 0.0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(l / r)
                    }
                }
                BinaryOp::Pow => Ok(l.powf(r)),
                _ => Err(format!("Cannot evaluate operator {:?}", op)),
            }
        }
        Expr::Unary {
            op: UnaryOp::Neg,
            expr,
        } => Ok(-evaluate(expr)?),
        _ => Err("Cannot evaluate this expression".to_string()),
    }
}

/// Parse a simple expression
pub fn parse_expression(input: &str) -> Result<Expr, peg::error::ParseError<peg::str::LineCol>> {
    functional_parser::expression(input)
}

/// Parse a complete program
pub fn parse_program(input: &str) -> Result<Program, peg::error::ParseError<peg::str::LineCol>> {
    functional_parser::program(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_parsing() {
        let result = parse_expression("42").unwrap();
        assert_eq!(result, Expr::Number(42));

        let result = parse_expression("-17").unwrap();
        assert_eq!(
            result,
            Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(Expr::Number(17))
            }
        );
    }

    #[test]
    fn test_binary_expression() {
        let result = parse_expression("2 + 3").unwrap();
        if let Expr::Binary { left, op, right } = result {
            assert_eq!(*left, Expr::Number(2));
            assert_eq!(op, BinaryOp::Add);
            assert_eq!(*right, Expr::Number(3));
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_operator_precedence() {
        let result = parse_expression("2 + 3 * 4").unwrap();
        // Should parse as 2 + (3 * 4)
        if let Expr::Binary { left, op, right } = result {
            assert_eq!(*left, Expr::Number(2));
            assert_eq!(op, BinaryOp::Add);

            if let Expr::Binary {
                left: rl,
                op: rop,
                right: rr,
            } = right.as_ref()
            {
                assert_eq!(rl.as_ref(), &Expr::Number(3));
                assert_eq!(*rop, BinaryOp::Mul);
                assert_eq!(rr.as_ref(), &Expr::Number(4));
            } else {
                panic!("Expected binary expression on right");
            }
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_evaluation() {
        let expr = parse_expression("2 + 3 * 4").unwrap();
        let result = evaluate(&expr).unwrap();
        assert_eq!(result, 14.0);

        let expr = parse_expression("(2 + 3) * 4").unwrap();
        let result = evaluate(&expr).unwrap();
        assert_eq!(result, 20.0);

        let expr = parse_expression("2 ** 3").unwrap();
        let result = evaluate(&expr).unwrap();
        assert_eq!(result, 8.0);
    }

    #[test]
    fn test_function_call() {
        let result = parse_expression("foo(1, 2, 3)").unwrap();
        if let Expr::Call { func, args } = result {
            assert_eq!(*func, Expr::Identifier("foo".to_string()));
            assert_eq!(args.len(), 3);
            assert_eq!(args[0], Expr::Number(1));
            assert_eq!(args[1], Expr::Number(2));
            assert_eq!(args[2], Expr::Number(3));
        } else {
            panic!("Expected function call");
        }
    }

    #[test]
    fn test_string_literals() {
        let result = parse_expression("\"hello world\"").unwrap();
        assert_eq!(result, Expr::String("hello world".to_string()));

        let result = parse_expression("\"escaped\\nnewline\"").unwrap();
        assert_eq!(result, Expr::String("escaped\nnewline".to_string()));
    }

    #[test]
    fn test_list_parsing() {
        let result = parse_expression("[1, 2, 3]").unwrap();
        assert_eq!(
            result,
            Expr::List(vec![Expr::Number(1), Expr::Number(2), Expr::Number(3)])
        );

        let result = parse_expression("[]").unwrap();
        assert_eq!(result, Expr::List(vec![]));
    }

    #[test]
    fn test_let_expression() {
        let result = parse_expression("let x = 5 in x + 1").unwrap();
        if let Expr::Let { bindings, body } = result {
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].0, "x");
            assert_eq!(bindings[0].1, Expr::Number(5));

            if let Expr::Binary { left, op, right } = &*body {
                assert_eq!(**left, Expr::Identifier("x".to_string()));
                assert_eq!(*op, BinaryOp::Add);
                assert_eq!(**right, Expr::Number(1));
            } else {
                panic!("Expected binary expression in body");
            }
        } else {
            panic!("Expected let expression");
        }
    }

    #[test]
    fn test_if_expression() {
        let result = parse_expression("if true then 1 else 2").unwrap();
        if let Expr::If {
            condition,
            then_branch,
            else_branch,
        } = result
        {
            assert_eq!(*condition, Expr::Bool(true));
            assert_eq!(*then_branch, Expr::Number(1));
            assert_eq!(
                else_branch.as_ref().map(|b| b.as_ref()),
                Some(&Expr::Number(2))
            );
        } else {
            panic!("Expected if expression");
        }
    }

    #[test]
    fn test_lambda_expression() {
        let result = parse_expression("\\x -> x + 1").unwrap();
        if let Expr::Lambda { params, body } = result {
            assert_eq!(params, vec!["x"]);
            if let Expr::Binary { left, op, right } = &*body {
                assert_eq!(**left, Expr::Identifier("x".to_string()));
                assert_eq!(*op, BinaryOp::Add);
                assert_eq!(**right, Expr::Number(1));
            } else {
                panic!("Expected binary expression in body");
            }
        } else {
            panic!("Expected lambda expression");
        }
    }

    #[test]
    fn test_error_reporting() {
        let result = parse_expression("2 + ");
        assert!(result.is_err());
    }
}
