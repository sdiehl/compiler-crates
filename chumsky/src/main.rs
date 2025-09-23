use chumsky::prelude::*;
use chumsky_example::{expr_parser, lexer, robust_parser, validated_parser};

fn main() {
    println!("=== Chumsky Parser Combinator Examples ===\n");

    // Basic expression parsing
    println!("1. Basic Expression Parsing:");
    let parser = expr_parser();
    let input = "2 + 3 * 4 - 1";
    let result = parser.parse(input);
    if !result.has_errors() {
        println!(
            "  Input: {}\n  AST: {:#?}",
            input,
            result.into_output().unwrap()
        );
    } else {
        println!("  Parse errors:");
        for err in result.into_errors() {
            println!("    {}", err);
        }
    }

    // Let bindings and function calls
    println!("\n2. Let Bindings and Function Calls:");
    let input = "let x = 5 in max(x, 10)";
    let result = parser.parse(input);
    if !result.has_errors() {
        println!(
            "  Input: {}\n  AST: {:#?}",
            input,
            result.into_output().unwrap()
        );
    } else {
        println!("  Parse errors:");
        for err in result.into_errors() {
            println!("    {}", err);
        }
    }

    // Lexer demonstration
    println!("\n3. Lexer with Spans:");
    let lexer = lexer();
    let input = "let x = 42 in x + 1";
    let result = lexer.parse(input);
    if !result.has_errors() {
        println!("  Input: {}", input);
        println!("  Tokens:");
        for (token, span) in result.into_output().unwrap() {
            println!("    {:?} at {:?}", token, span);
        }
    } else {
        println!("  Lexer errors:");
        for err in result.into_errors() {
            println!("    {}", err);
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
        let result = robust.parse(input);
        if let Some(exprs) = result.output() {
            println!("    Recovered {} expressions", exprs.len());
            for (i, expr) in exprs.iter().enumerate() {
                println!("      [{}]: {:?}", i, expr);
            }
        }
        if result.has_errors() {
            let errors = result.into_errors();
            println!("    Parse failed with {} errors", errors.len());
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
        let result = validated.parse(input);
        if !result.has_errors() {
            println!("Valid: {:?}", result.into_output().unwrap());
        } else {
            print!("Invalid: ");
            for err in result.into_errors() {
                print!("{} ", err);
            }
            println!();
        }
    }
}
