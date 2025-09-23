use logos_example::{
    parse_expression, tokenize, tokenize_with_errors, SourceTracker, Token, TokenStream,
};

fn main() {
    println!("=== Basic Tokenization ===");
    let source = r#"
fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    fibonacci(n - 1) + fibonacci(n - 2)
}
"#;

    let tokens = tokenize(source);
    println!("Tokenizing fibonacci function:");
    for (token, span) in &tokens[..20] {
        // First 20 tokens
        let text = &source[span.clone()];
        println!("  {:?} => '{}'", token, text);
    }

    println!();
    println!("=== Error Handling ===");
    let invalid_source = "let x = 42 @ invalid $ symbols";
    let (tokens, errors) = tokenize_with_errors(invalid_source);

    println!("Valid tokens:");
    for (token, span) in &tokens {
        let text = &invalid_source[span.clone()];
        println!("  {:?} => '{}'", token, text);
    }

    println!();
    println!("Lexical errors at positions:");
    for span in &errors {
        println!("  {} => '{}'", span.start, &invalid_source[span.clone()]);
    }

    println!();
    println!("=== Source Location Tracking ===");
    let multi_line = r#"fn main() {
    let x = 42;
    let y = x * 2;
    println!("{}", y);
}"#;

    let tracker = SourceTracker::new(multi_line);
    let tokens = tokenize(multi_line);

    println!("Token locations:");
    for (token, span) in tokens.iter().skip(5).take(10) {
        let loc = tracker.location(span.start);
        println!("  {:?} at line {}, column {}", token, loc.line, loc.column);
    }

    println!();
    println!("=== Stream Processing ===");
    demonstrate_token_stream();

    println!();
    println!("=== Performance Test ===");
    benchmark_lexer();

    println!();
    println!("=== Expression Parsing ===");
    let expr = "x + 42 * (y - 3) / 2";
    let expr_tokens = parse_expression(expr);
    println!("Expression: {}", expr);
    println!("Tokens: {:?}", expr_tokens);

    println!();
    println!("=== Complex Token Patterns ===");
    demonstrate_complex_tokens();

    println!();
    println!("=== Real Compiler Example ===");
    demonstrate_compiler_lexer();
}

fn demonstrate_token_stream() {
    let source = "let mut x = 42;";
    let mut stream = TokenStream::new(source);

    println!("Token stream processing:");
    while let Some(result) = stream.next_token() {
        if let Ok(token) = result {
            let span = stream.span();
            println!("  {:?} at {:?}", token, span);
        }
    }
}

fn benchmark_lexer() {
    let large_source = r#"
        fn complex_function() {
            let data = vec![1, 2, 3, 4, 5];
            for i in 0..100 {
                if i % 2 == 0 {
                    println!("Even: {}", i);
                } else {
                    println!("Odd: {}", i);
                }
            }
        }
    "#
    .repeat(100);

    let start = std::time::Instant::now();
    let tokens = tokenize(&large_source);
    let elapsed = start.elapsed();

    println!(
        "Lexed {} tokens from {} bytes in {:?}",
        tokens.len(),
        large_source.len(),
        elapsed
    );
    println!(
        "Throughput: {:.2} MB/s",
        large_source.len() as f64 / elapsed.as_secs_f64() / 1_000_000.0
    );
}

fn demonstrate_complex_tokens() {
    let source = r#"
// Single-line comment
/* Multi-line 
   comment */
"string with \"escapes\""
'c'
123
456.789
0xFF
0b1010
"#;

    let tokens = tokenize(source);
    println!("Complex token examples:");
    for (token, span) in &tokens {
        match token {
            Token::String(s) => println!("  String literal: \"{}\"", s),
            Token::Integer(Some(n)) => println!("  Integer: {}", n),
            Token::Float(Some(f)) => println!("  Float: {}", f),
            Token::Char => println!("  Character: {}", &source[span.clone()]),
            _ => {}
        }
    }
}

fn demonstrate_compiler_lexer() {
    let program = r#"
mod ast {
    pub struct Node {
        kind: NodeKind,
        span: Span,
    }

    pub enum NodeKind {
        BinaryOp { op: Operator, left: Box<Node>, right: Box<Node> },
        UnaryOp { op: Operator, expr: Box<Node> },
        Literal(Value),
        Identifier(String),
    }
}

impl Parser {
    fn parse_expression(&mut self) -> Result<Node, Error> {
        let mut left = self.parse_primary()?;
        
        while let Some(op) = self.match_operator() {
            let right = self.parse_primary()?;
            left = Node::binary(op, left, right);
        }
        
        Ok(left)
    }
}
"#;

    println!("Lexing a real compiler module:");
    let tracker = SourceTracker::new(program);
    let (tokens, errors) = tokenize_with_errors(program);

    // Show token statistics
    let mut keyword_count = 0;
    let mut identifier_count = 0;
    let mut operator_count = 0;

    for (token, _) in &tokens {
        match token {
            Token::Function
            | Token::Let
            | Token::Const
            | Token::If
            | Token::Else
            | Token::While
            | Token::For
            | Token::Return
            | Token::Struct
            | Token::Enum
            | Token::Impl
            | Token::Trait
            | Token::Pub
            | Token::Mod
            | Token::Use
            | Token::Mut => keyword_count += 1,
            Token::Identifier(_) => identifier_count += 1,
            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Equal
            | Token::Less
            | Token::Greater
            | Token::Arrow => operator_count += 1,
            _ => {}
        }
    }

    println!("  Total tokens: {}", tokens.len());
    println!("  Keywords: {}", keyword_count);
    println!("  Identifiers: {}", identifier_count);
    println!("  Operators: {}", operator_count);
    println!("  Lexical errors: {}", errors.len());

    // Show first few tokens with location info
    println!();
    println!("First 15 tokens with locations:");
    for (token, span) in tokens.iter().take(15) {
        let loc = tracker.location(span.start);
        let text = &program[span.clone()];
        println!(
            "  Line {}, Col {}: {:?} => '{}'",
            loc.line, loc.column, token, text
        );
    }
}
