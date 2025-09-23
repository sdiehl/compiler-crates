use peg_example::{parse_expression, parse_program, Statement};

fn main() {
    println!("PEG Parser Examples");
    println!("==================");

    // Basic expression parsing
    println!("\n1. Basic Expression Parsing:");
    let expressions = vec![
        "42",
        "3.14",
        "true",
        "\"hello world\"",
        "x",
        "2 + 3 * 4",
        "(1 + 2) * 3",
        "x + y - z",
        "pow(2, 3)",
        "not true and false",
        "x == 42 or y > 10",
    ];

    for expr_str in expressions {
        match parse_expression(expr_str) {
            Ok(expr) => println!("  '{}' -> {:?}", expr_str, expr),
            Err(e) => println!("  '{}' -> Error: {}", expr_str, e),
        }
    }

    // Lambda expressions
    println!("\n2. Lambda Expressions:");
    let lambda_examples = vec!["\\x -> x + 1", "\\x y -> x * y", "\\f x -> f(f(x))"];

    for lambda_str in lambda_examples {
        match parse_expression(lambda_str) {
            Ok(expr) => println!("  '{}' -> {:?}", lambda_str, expr),
            Err(e) => println!("  '{}' -> Error: {}", lambda_str, e),
        }
    }

    // Conditional expressions
    println!("\n3. Conditional Expressions:");
    let conditional_examples = vec![
        "if x > 0 then x else -x",
        "if true then 1 else 2",
        "if x == y then \"equal\" else \"different\"",
    ];

    for cond_str in conditional_examples {
        match parse_expression(cond_str) {
            Ok(expr) => println!("  '{}' -> {:?}", cond_str, expr),
            Err(e) => println!("  '{}' -> Error: {}", cond_str, e),
        }
    }

    // Let expressions
    println!("\n4. Let Expressions:");
    let let_examples = vec![
        "let x = 42 in x + 1",
        "let x = 1, y = 2 in x + y",
        "let double = \\x -> x * 2 in double(5)",
    ];

    for let_str in let_examples {
        match parse_expression(let_str) {
            Ok(expr) => println!("  '{}' -> {:?}", let_str, expr),
            Err(e) => println!("  '{}' -> Error: {}", let_str, e),
        }
    }

    // List and record expressions
    println!("\n5. Data Structure Expressions:");
    let data_examples = vec![
        "[1, 2, 3]",
        "[\"a\", \"b\", \"c\"]",
        "{ x: 42, y: \"hello\" }",
        "{ name: \"Alice\", age: 30, active: true }",
    ];

    for data_str in data_examples {
        match parse_expression(data_str) {
            Ok(expr) => println!("  '{}' -> {:?}", data_str, expr),
            Err(e) => println!("  '{}' -> Error: {}", data_str, e),
        }
    }

    // Full program parsing
    println!("\n6. Program Parsing:");
    let program_text = r#"
        let factorial = \n ->
            if n <= 1 then 1
            else n * factorial(n - 1)

        let main = factorial(5)

        let fibonacci = \n ->
            if n <= 1 then n
            else fibonacci(n - 1) + fibonacci(n - 2)

        let result = fibonacci(10)
    "#;

    match parse_program(program_text) {
        Ok(program) => {
            println!("  Parsed program successfully:");
            for statement in &program.statements {
                match statement {
                    Statement::Definition { name, value } => {
                        println!("    def {} = {:?}", name, value);
                    }
                    Statement::Expression(expr) => {
                        println!("    expr: {:?}", expr);
                    }
                    Statement::TypeDef { name, constructors } => {
                        println!("    type {} = {:?}", name, constructors);
                    }
                }
            }
        }
        Err(e) => println!("  Program parse error: {}", e),
    }

    // Error demonstration
    println!("\n7. Error Handling:");
    let error_examples = vec![
        "2 +",          // incomplete expression
        "let x = in y", // missing value
        "if x then",    // incomplete if
        "\\x ->",       // incomplete lambda
        "{ x: }",       // incomplete record
    ];

    for error_str in error_examples {
        match parse_expression(error_str) {
            Ok(expr) => println!("  '{}' -> Unexpected success: {:?}", error_str, expr),
            Err(e) => println!("  '{}' -> Error (expected): {}", error_str, e),
        }
    }

    // Precedence demonstration
    println!("\n8. Operator Precedence:");
    let precedence_examples = vec![
        "2 + 3 * 4",          // should be 2 + (3 * 4)
        "2 * 3 + 4",          // should be (2 * 3) + 4
        "2 ** 3 ** 2",        // should be 2 ** (3 ** 2) (right-associative)
        "not true and false", // should be (not true) and false
        "x == y and z > 0",   // should be (x == y) and (z > 0)
    ];

    for prec_str in precedence_examples {
        match parse_expression(prec_str) {
            Ok(expr) => println!("  '{}' -> {:?}", prec_str, expr),
            Err(e) => println!("  '{}' -> Error: {}", prec_str, e),
        }
    }

    println!("\nAll examples completed!");
}
