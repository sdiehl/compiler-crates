// Import Rule directly from the generated module
use pest_example::{GrammarParser, JsonValue, LiteralValue, Rule, Statement, Token};

fn main() {
    println!("=== Pest Parser Examples ===\n");

    // Expression parsing demonstration
    expression_demo();

    // Calculator with precedence demonstration
    calculator_demo();

    // JSON parsing demonstration
    json_demo();

    // Programming language parsing demonstration
    language_demo();

    // Token stream demonstration
    token_demo();

    // Debug and utility features demonstration
    debug_demo();

    println!("\n=== All demonstrations completed ===");
}

fn expression_demo() {
    println!("--- Expression Parsing ---");

    let expressions = [
        "2 + 3",
        "x * y + z",
        "a + b * c ^ d",
        "(x + y) * (z - w)",
        "foo + bar * 2.5",
    ];

    for expr_str in &expressions {
        match GrammarParser::parse_expression(expr_str) {
            Ok(expr) => {
                println!("Expression: {} -> AST: {:?}", expr_str, expr);
                let identifiers = GrammarParser::extract_identifiers(&expr);
                if !identifiers.is_empty() {
                    println!("  Identifiers found: {:?}", identifiers);
                }
            }
            Err(e) => println!("Failed to parse '{}': {}", expr_str, e),
        }
    }
    println!();
}

fn calculator_demo() {
    println!("--- Calculator with Precedence ---");

    let calculations = [
        "2 + 3 * 4",
        "(2 + 3) * 4",
        "2 ^ 3 ^ 2",
        "10 / 2 + 3",
        "-5 + 3",
        "2 * -3 + 4",
        "1 + 2 * 3 ^ 2",
    ];

    for calc in &calculations {
        match GrammarParser::parse_calculation(calc) {
            Ok(result) => println!("Calculation: {} = {}", calc, result),
            Err(e) => println!("Failed to calculate '{}': {}", calc, e),
        }
    }
    println!();
}

fn json_demo() {
    println!("--- JSON Parsing ---");

    let json_examples = [
        r#"null"#,
        r#"true"#,
        r#"42"#,
        r#""hello world""#,
        r#"[1, 2, 3]"#,
        r#"{"name": "John", "age": 30}"#,
        r#"{
            "user": {
                "id": 123,
                "name": "Alice",
                "active": true,
                "tags": ["developer", "rust"]
            },
            "metadata": null
        }"#,
    ];

    for json_str in &json_examples {
        match GrammarParser::parse_json(json_str) {
            Ok(json_value) => {
                println!(
                    "JSON input: {}",
                    json_str.chars().take(50).collect::<String>()
                );
                print_json_value(&json_value, 2);
                println!();
            }
            Err(e) => println!("Failed to parse JSON: {}", e),
        }
    }
    println!();
}

fn print_json_value(value: &JsonValue, indent: usize) {
    let spaces = " ".repeat(indent);
    match value {
        JsonValue::Object(obj) => {
            println!("{}Object with {} fields:", spaces, obj.len());
            for (key, val) in obj {
                println!("{}  {}: ", spaces, key);
                print_json_value(val, indent + 4);
            }
        }
        JsonValue::Array(arr) => {
            println!("{}Array with {} elements:", spaces, arr.len());
            for (i, val) in arr.iter().enumerate() {
                println!("{}  [{}]: ", spaces, i);
                print_json_value(val, indent + 4);
            }
        }
        JsonValue::String(s) => println!("{}String: \"{}\"", spaces, s),
        JsonValue::Number(n) => println!("{}Number: {}", spaces, n),
        JsonValue::Boolean(b) => println!("{}Boolean: {}", spaces, b),
        JsonValue::Null => println!("{}Null", spaces),
    }
}

fn language_demo() {
    println!("--- Programming Language Parsing ---");

    let program_examples = [
        r#"
            x = 42;
        "#,
        r#"
            if x > 0 {
                y = x + 1;
            } else {
                y = 0;
            }
        "#,
        r#"
            while count < 10 {
                count = count + 1;
            }
        "#,
        r#"
            fn factorial(n: int) -> int {
                if n <= 1 {
                    result = 1;
                } else {
                    result = n * factorial(n - 1);
                }
            }
        "#,
        r#"
            fn main() -> int {
                x = 5;
                y = 10;

                if x < y {
                    z = x + y;
                } else {
                    z = x - y;
                }

                while z > 0 {
                    z = z - 1;
                }
            }
        "#,
    ];

    for (i, program_str) in program_examples.iter().enumerate() {
        println!("Program example {}:", i + 1);
        match GrammarParser::parse_program(program_str) {
            Ok(program) => {
                println!("  Parsed {} statements:", program.statements.len());
                for (j, stmt) in program.statements.iter().enumerate() {
                    print_statement(stmt, j, 4);
                }
            }
            Err(e) => println!("  Failed to parse: {}", e),
        }
        println!();
    }
}

fn print_statement(stmt: &Statement, index: usize, indent: usize) {
    let spaces = " ".repeat(indent);
    match stmt {
        Statement::Assignment { name, .. } => {
            println!("{}[{}] Assignment to '{}'", spaces, index, name);
        }
        Statement::If { .. } => {
            println!("{}[{}] If statement", spaces, index);
        }
        Statement::While { .. } => {
            println!("{}[{}] While loop", spaces, index);
        }
        Statement::Function {
            name,
            parameters,
            return_type,
            ..
        } => {
            println!(
                "{}[{}] Function '{}' with {} parameters -> {}",
                spaces,
                index,
                name,
                parameters.len(),
                return_type
            );
        }
        Statement::Expression(_) => {
            println!("{}[{}] Expression statement", spaces, index);
        }
        Statement::Block(stmts) => {
            println!(
                "{}[{}] Block with {} statements",
                spaces,
                index,
                stmts.len()
            );
        }
    }
}

fn token_demo() {
    println!("--- Token Stream Parsing ---");

    let token_examples = [
        "if x == 42 { return true; }",
        "let mut count += 1;",
        r#"name = "Alice"; age = 25;"#,
        "fn main() -> bool { true }",
        "while x <= 100 && y >= 0 { x++; y--; }",
    ];

    for input in &token_examples {
        println!("Input: {}", input);
        match GrammarParser::parse_tokens(input) {
            Ok(tokens) => {
                println!("  Tokens ({}):", tokens.len());
                for (i, token) in tokens.iter().enumerate() {
                    print_token(token, i);
                }
            }
            Err(e) => println!("  Failed to tokenize: {}", e),
        }
        println!();
    }
}

fn print_token(token: &Token, index: usize) {
    match token {
        Token::Keyword(kw) => println!("    [{}] Keyword: {}", index, kw),
        Token::Operator(op) => println!("    [{}] Operator: {}", index, op),
        Token::Punctuation(p) => println!("    [{}] Punctuation: {}", index, p),
        Token::Literal(lit) => match lit {
            LiteralValue::String(s) => println!("    [{}] String: \"{}\"", index, s),
            LiteralValue::Number(n) => println!("    [{}] Number: {}", index, n),
            LiteralValue::Boolean(b) => println!("    [{}] Boolean: {}", index, b),
        },
        Token::Identifier(id) => println!("    [{}] Identifier: {}", index, id),
    }
}

fn debug_demo() {
    println!("--- Debug and Utility Features ---");

    // Test parsing capability
    let test_inputs = [
        ("expression", "x + y"),
        ("expression", "2 +"), // Invalid
        ("json_value", r#"{"valid": true}"#),
        ("json_value", r#"{"invalid": }"#), // Invalid
    ];

    for (rule_name, input) in &test_inputs {
        let rule = match *rule_name {
            "expression" => Rule::expression,
            "json_value" => Rule::json_value,
            _ => continue,
        };

        let can_parse = GrammarParser::can_parse(rule, input);
        println!("Can parse '{}' as {}: {}", input, rule_name, can_parse);
    }

    println!("\nDebug parse tree for '2 + 3 * 4':");
    if let Err(e) = GrammarParser::debug_parse(Rule::expression, "2 + 3 * 4") {
        println!("Debug parse failed: {}", e);
    }
}
