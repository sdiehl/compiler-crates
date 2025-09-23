use codespan::{ByteIndex, Span};
use codespan_example::{
    demonstrate_line_offsets, demonstrate_span_arithmetic, track_utf8_positions, Lexer, SourceFile,
    SpanManager, TokenKind,
};

fn main() {
    println!("Codespan Examples");
    println!("=================\n");

    // Basic span tracking
    println!("1. Basic Span Tracking:");
    let source = "let x = 42;\nlet y = x + 1;\nprint(y);";
    let file = SourceFile::new("example.lang".to_string(), source.to_string());

    println!("Source file: {}", file.name());
    println!("Contents:\n{}\n", file.contents());

    // Show line and column for various positions
    let positions = vec![
        ByteIndex::from(0),  // 'l' in first 'let'
        ByteIndex::from(4),  // 'x'
        ByteIndex::from(12), // 'l' in second 'let'
        ByteIndex::from(27), // 'p' in 'print'
    ];

    for pos in positions {
        let loc = file.location(pos);
        println!("  Byte {} -> {}", pos.to_usize(), loc);
    }

    // Span slicing
    println!("\n2. Span Slicing:");
    let span1 = Span::new(ByteIndex::from(0), ByteIndex::from(3)); // "let"
    let span2 = Span::new(ByteIndex::from(4), ByteIndex::from(5)); // "x"
    let span3 = Span::new(ByteIndex::from(8), ByteIndex::from(10)); // "42"

    println!("  Span [0, 3): '{}'", file.slice(span1));
    println!("  Span [4, 5): '{}'", file.slice(span2));
    println!("  Span [8, 10): '{}'", file.slice(span3));

    // Multi-file management
    println!("\n3. Multi-File Span Management:");
    let mut manager = SpanManager::new();

    let main_file = manager.add_file(
        "main.lang".to_string(),
        r#"import "lib";

function main() {
    let result = add(10, 20);
    print(result);
}"#
        .to_string(),
    );

    let lib_file = manager.add_file(
        "lib.lang".to_string(),
        r#"function add(a, b) {
    return a + b;
}

function multiply(a, b) {
    return a * b;
}"#
        .to_string(),
    );

    println!("  Registered files:");
    if let Some(main) = manager.get_file(main_file) {
        println!("    - {} ({} bytes)", main.name(), main.contents().len());
    }
    if let Some(lib) = manager.get_file(lib_file) {
        println!("    - {} ({} bytes)", lib.name(), lib.contents().len());
    }

    // Lexer with span tracking
    println!("\n4. Lexer with Span Tracking:");
    let code = "let x = 42 + 3 * 2;";
    println!("  Input: {}", code);

    let file_id = manager.add_file("lexer_test.lang".to_string(), code.to_string());
    let mut lexer = Lexer::new(code.to_string(), file_id);
    let tokens = lexer.tokenize();

    println!("  Tokens:");
    if let Some(file) = manager.get_file(file_id) {
        for token in &tokens {
            let text = file.slice(token.span);
            let loc = file.location(token.span.start());
            println!("    {:?} '{}' at {}", token.kind, text, loc);
        }
    }

    // Span arithmetic
    println!("\n5. Span Arithmetic:");
    demonstrate_span_arithmetic();
    println!("  ✓ Span arithmetic operations verified");

    // Line offset calculations
    println!("\n6. Line Offset Calculations:");
    demonstrate_line_offsets();
    println!("  ✓ Line offset operations verified");

    // UTF-8 aware tracking
    println!("\n7. UTF-8 Aware Position Tracking:");
    let utf8_text = "hello 世界! 🦀";
    let positions = track_utf8_positions(utf8_text);

    println!("  Text: '{}'", utf8_text);
    println!("  Character positions:");
    for (ch, pos) in &positions[..positions.len().min(10)] {
        println!("    '{}' at byte {}", ch, pos.to_usize());
    }

    // Complex tokenization example
    println!("\n8. Complex Tokenization:");
    let complex_code = r#"function fibonacci(n) {
    if n <= 1 {
        return n;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}"#;

    let fib_file = manager.add_file("fibonacci.lang".to_string(), complex_code.to_string());
    let mut lexer = Lexer::new(complex_code.to_string(), fib_file);
    let tokens = lexer.tokenize();

    println!("  Source:\n{}", complex_code);
    println!("\n  Token summary:");

    let mut keywords = 0;
    let mut identifiers = 0;
    let mut operators = 0;
    let mut delimiters = 0;
    let mut numbers = 0;

    for token in &tokens {
        match &token.kind {
            TokenKind::Keyword(_) => keywords += 1,
            TokenKind::Identifier(_) => identifiers += 1,
            TokenKind::Operator(_) => operators += 1,
            TokenKind::Delimiter(_) => delimiters += 1,
            TokenKind::Number(_) => numbers += 1,
            _ => {}
        }
    }

    println!("    Keywords: {}", keywords);
    println!("    Identifiers: {}", identifiers);
    println!("    Operators: {}", operators);
    println!("    Delimiters: {}", delimiters);
    println!("    Numbers: {}", numbers);

    // Span merging
    println!("\n9. Span Merging:");
    let span_a = manager.create_span(ByteIndex::from(10), ByteIndex::from(20));
    let span_b = manager.create_span(ByteIndex::from(15), ByteIndex::from(25));
    let merged = manager.merge_spans(span_a, span_b);

    println!(
        "  Span A: [{}, {})",
        span_a.start().to_usize(),
        span_a.end().to_usize()
    );
    println!(
        "  Span B: [{}, {})",
        span_b.start().to_usize(),
        span_b.end().to_usize()
    );
    println!(
        "  Merged: [{}, {})",
        merged.start().to_usize(),
        merged.end().to_usize()
    );

    // Error location reporting
    println!("\n10. Error Location Reporting:");
    let error_code = "let x = ;\nlet y = 10;";
    let err_file = manager.add_file("error.lang".to_string(), error_code.to_string());

    if let Some(file) = manager.get_file(err_file) {
        let error_pos = ByteIndex::from(8); // Position of ';'
        let location = file.location(error_pos);

        println!("  Source with error:");
        println!("{}", error_code);
        println!("\n  Error at {}: unexpected ';'", location);
        println!("  Expected expression after '='");
    }

    println!("\nAll examples completed!");
}
