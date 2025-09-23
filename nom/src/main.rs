use nom_example::{
    expression, parse_binary_header, parse_config, streaming_parser, string_literal,
};

fn main() {
    println!("=== Nom Parser Combinator Examples ===\n");

    // Basic expression parsing
    println!("1. Expression Parsing:");
    let inputs = vec!["2 + 3 * 4", "10 - 5 - 2", "(1 + 2) * 3"];

    for input in inputs {
        match expression(input) {
            Ok((remaining, expr)) => {
                println!("  Input: {}", input);
                println!("    Parsed: {:?}", expr);
                println!("    Remaining: {:?}", remaining);
            }
            Err(e) => println!("  Parse error: {:?}", e),
        }
    }

    // Function calls and arrays
    println!("\n2. Function Calls and Arrays:");
    let inputs = vec![
        "max(1, 2, 3)",
        "[1, 2, 3, 4, 5]",
        "process([\"a\", \"b\"], 42)",
    ];

    for input in inputs {
        match expression(input) {
            Ok((_, expr)) => {
                println!("  Input: {}", input);
                println!("    AST: {:#?}", expr);
            }
            Err(e) => println!("  Parse error: {:?}", e),
        }
    }

    // String parsing with escapes
    println!("\n3. String Literals:");
    let strings = vec![
        r#""hello world""#,
        r#""line 1\nline 2""#,
        r#""quote: \"test\"""#,
    ];

    for input in strings {
        match string_literal(input) {
            Ok((_, s)) => {
                println!("  Input: {}", input);
                println!("    Parsed: {}", s);
            }
            Err(e) => println!("  Parse error: {:?}", e),
        }
    }

    // Configuration file parsing
    println!("\n4. Configuration File:");
    let config = r#"
[database]
host = "localhost"
port = 5432
ssl = true

[cache]
enabled = false
servers = ["cache1", "cache2", "cache3"]
"#;

    match parse_config(config.trim()) {
        Ok((_, config)) => {
            println!("  Parsed configuration:");
            for section in config.sections {
                println!("    [{}]", section.name);
                for (key, value) in section.entries {
                    println!("      {} = {:?}", key, value);
                }
            }
        }
        Err(e) => println!("  Parse error: {:?}", e),
    }

    // Streaming parser
    println!("\n5. Streaming Parser:");
    let input = "1 + 2; 3 * 4; 5 - 1";
    match streaming_parser(input) {
        Ok((_, exprs)) => {
            println!("  Input: {}", input);
            println!("  Parsed {} expressions:", exprs.len());
            for (i, expr) in exprs.iter().enumerate() {
                println!("    [{}]: {:?}", i, expr);
            }
        }
        Err(e) => println!("  Parse error: {:?}", e),
    }

    // Binary parsing example
    println!("\n6. Binary Format:");
    let binary_data = b"MAGIC\x01\x00\x00\x00\x00\x00\x00\x42";
    match parse_binary_header(binary_data) {
        Ok((remaining, (version, flags))) => {
            println!("  Magic header found!");
            println!("    Version: {}", version);
            println!("    Flags: 0x{:08x}", flags);
            println!("    Remaining bytes: {}", remaining.len());
        }
        Err(e) => println!("  Parse error: {:?}", e),
    }
}
