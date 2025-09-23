use chumsky::prelude::*;
use chumsky_example::{expr_parser, lexer, robust_parser, validated_parser};

fn main() {
    println!("=== Chumsky Parser Combinator Examples ===\n");

    // Basic expression parsing
    println!("1. Basic Expression Parsing:");
    let parser = expr_parser();
    let input = "2 + 3 * 4 - 1";
    match parser.parse(input) {
        Ok(expr) => println!("  Input: {}\n  AST: {:#?}", input, expr),
        Err(errors) => {
            println!("  Parse errors:");
            for err in errors {
                println!("    {}", err);
            }
        }
    }

    // Let bindings and function calls
    println!("\n2. Let Bindings and Function Calls:");
    let input = "let x = 5 in max(x, 10)";
    match parser.parse(input) {
        Ok(expr) => println!("  Input: {}\n  AST: {:#?}", input, expr),
        Err(errors) => {
            println!("  Parse errors:");
            for err in errors {
                println!("    {}", err);
            }
        }
    }

    // Lexer demonstration
    println!("\n3. Lexer with Spans:");
    let lexer = lexer();
    let input = "let x = 42 in x + 1";
    match lexer.parse(input) {
        Ok(tokens) => {
            println!("  Input: {}", input);
            println!("  Tokens:");
            for (token, span) in tokens {
                println!("    {:?} at {:?}", token, span);
            }
        }
        Err(errors) => {
            println!("  Lexer errors:");
            for err in errors {
                println!("    {}", err);
            }
        }
    }

    // Error recovery
    println!("\n4. Error Recovery:");
    let robust = robust_parser();
    let inputs = vec![
        "1 + 2; 3 * 4",      // Valid
        "1 + ; 3 * 4",       // Missing operand
        "1 + (2 * 3; 4 + 5", // Missing closing paren
    ];

    for input in inputs {
        println!("  Input: {}", input);
        match robust.parse(input) {
            Ok(exprs) => {
                println!("    Recovered {} expressions", exprs.len());
                for (i, expr) in exprs.iter().enumerate() {
                    println!("      [{}]: {:?}", i, expr);
                }
            }
            Err(errors) => {
                println!("    Parse failed with {} errors", errors.len());
            }
        }
    }

    // Validation
    println!("\n5. Custom Validation:");
    let validated = validated_parser();
    let inputs = vec![
        "42",
        "3.14159",
        "999999999999999999999999999999", // Too large
        "valid_identifier",
    ];

    for input in inputs {
        print!("  Input: {} -> ", input);
        match validated.parse(input) {
            Ok(expr) => println!("Valid: {:?}", expr),
            Err(errors) => {
                print!("Invalid: ");
                for err in errors {
                    print!("{} ", err);
                }
                println!();
            }
        }
    }
}
