use nom_locate_example::{Expr, LocatedLexer, Parser, Spanned, TokenKind};

fn main() {
    println!("=== nom-locate: Location-Aware Parsing ===");
    println!();

    // Example 1: Basic expression parsing with location tracking
    demo_expression_parsing();
    println!();

    // Example 2: Error reporting with precise locations
    demo_error_reporting();
    println!();

    // Example 3: Token-level location tracking
    demo_token_locations();
    println!();

    // Example 4: Multi-line input handling
    demo_multiline_parsing();
    println!();

    // Example 5: Complex expression with nested locations
    demo_complex_expression();
}

fn demo_expression_parsing() {
    println!("--- Expression Parsing with Locations ---");

    let expressions = vec![
        "42",
        "x + y",
        "add(1, 2)",
        "let x = 5 in x * 2",
        "(a + b) * c",
    ];

    for expr in expressions {
        println!("Parsing: {}", expr);
        match Parser::parse_expression(expr) {
            Ok(parsed) => {
                println!("  AST: {:?}", parsed.node);
                println!(
                    "  Location: line {}, col {} to col {}",
                    parsed.span.start.line, parsed.span.start.column, parsed.span.end.column
                );
                println!(
                    "  Byte range: {}..{}",
                    parsed.span.start.offset, parsed.span.end.offset
                );
            }
            Err(e) => {
                println!(
                    "  Error: {} at line {}, col {}",
                    e.message, e.location.line, e.location.column
                );
            }
        }
        println!();
    }
}

fn demo_error_reporting() {
    println!("--- Error Reporting with Precise Locations ---");

    let invalid_expressions = vec!["2 +", "let x = in y", "add(1, 2", "123abc", "x y z"];

    for expr in invalid_expressions {
        println!("Parsing invalid: {}", expr);
        match Parser::parse_expression(expr) {
            Ok(_) => println!("  Unexpected success!"),
            Err(e) => {
                println!("  Error: {}", e.message);
                println!(
                    "  At line {}, column {}",
                    e.location.line, e.location.column
                );
                println!("  Byte offset: {}", e.location.offset);

                // Show the error context
                if let Some(line_content) = Parser::get_line_content(expr, e.location.line) {
                    println!("  Context: {}", line_content);
                    let pointer = " ".repeat(e.location.column.saturating_sub(1)) + "^";
                    println!("           {}", pointer);
                }
            }
        }
        println!();
    }
}

fn demo_token_locations() {
    println!("--- Token-Level Location Tracking ---");

    let code = "let add = fn(x, y) x + y";
    println!("Tokenizing: {}", code);

    let mut lexer = LocatedLexer::new(code);
    match lexer.tokenize() {
        Ok(tokens) => {
            println!("Tokens:");
            for (i, token) in tokens.iter().enumerate() {
                if token.kind == TokenKind::Eof {
                    continue;
                }
                println!(
                    "  {}: {:?} '{}' at line {}, col {} (offset {})",
                    i,
                    token.kind,
                    token.text,
                    token.location.line,
                    token.location.column,
                    token.location.offset
                );
            }
        }
        Err(e) => {
            println!(
                "Tokenization error: {} at line {}, col {}",
                e.message, e.location.line, e.location.column
            );
        }
    }
}

fn demo_multiline_parsing() {
    println!("--- Multi-line Input Handling ---");

    let multiline_code = r#"let factorial = fn(n)
    if n <= 1
    then 1
    else n * factorial(n - 1)

let result = factorial(5)"#;

    println!("Code:");
    for (i, line) in multiline_code.lines().enumerate() {
        println!("  {}: {}", i + 1, line);
    }
    println!();

    // Tokenize to see location tracking across lines
    let mut lexer = LocatedLexer::new(multiline_code);
    match lexer.tokenize() {
        Ok(tokens) => {
            println!("Key tokens with locations:");
            for token in &tokens {
                match &token.kind {
                    TokenKind::Keyword(kw)
                        if kw == "let" || kw == "if" || kw == "then" || kw == "else" =>
                    {
                        println!(
                            "  {} '{}' at line {}, col {}",
                            kw, token.text, token.location.line, token.location.column
                        );
                    }
                    TokenKind::Identifier
                        if token.text == "factorial" || token.text == "result" =>
                    {
                        println!(
                            "  identifier '{}' at line {}, col {}",
                            token.text, token.location.line, token.location.column
                        );
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            println!(
                "Error: {} at line {}, col {}",
                e.message, e.location.line, e.location.column
            );
        }
    }
}

fn demo_complex_expression() {
    println!("--- Complex Expression with Nested Locations ---");

    let complex_expr = "let x = add(mul(2, 3), 4) in x + 1";
    println!("Expression: {}", complex_expr);

    match Parser::parse_expression(complex_expr) {
        Ok(parsed) => {
            println!("Successfully parsed!");
            print_expression_tree(&parsed, 0);
        }
        Err(e) => {
            println!(
                "Parse error: {} at line {}, col {}",
                e.message, e.location.line, e.location.column
            );
        }
    }
}

fn print_expression_tree(expr: &Spanned<Expr>, indent: usize) {
    let indent_str = "  ".repeat(indent);
    let location_info = format!(
        " ({}:{}-{}:{})",
        expr.span.start.line, expr.span.start.column, expr.span.end.line, expr.span.end.column
    );

    match &expr.node {
        Expr::Number(n) => {
            println!("{}Number({}){}", indent_str, n, location_info);
        }
        Expr::Identifier(name) => {
            println!("{}Identifier({}){}", indent_str, name, location_info);
        }
        Expr::Binary { left, op, right } => {
            println!("{}Binary({:?}){}", indent_str, op, location_info);
            print_expression_tree(left, indent + 1);
            print_expression_tree(right, indent + 1);
        }
        Expr::Call { func, args } => {
            println!("{}Call{}", indent_str, location_info);
            println!("{}  func:", indent_str);
            print_expression_tree(func, indent + 2);
            println!("{}  args:", indent_str);
            for arg in args {
                print_expression_tree(arg, indent + 2);
            }
        }
        Expr::Let { name, value, body } => {
            println!("{}Let({}){}", indent_str, name, location_info);
            println!("{}  value:", indent_str);
            print_expression_tree(value, indent + 2);
            println!("{}  body:", indent_str);
            print_expression_tree(body, indent + 2);
        }
    }
}
