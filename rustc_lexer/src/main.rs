use rustc_lexer::{LiteralKind, TokenKind};
use rustc_lexer_example::{
    cook_lexer_literal, describe_token, is_comment, is_whitespace, tokenize_and_validate, Lexer,
    ParsedLiteral,
};

fn main() {
    println!("=== Basic Rust Tokenization ===");
    let source = r#"
fn calculate(x: i32, y: f64) -> f64 {
    let result = x as f64 + y * 2.0;
    result
}
"#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    println!("Tokens from Rust function:");
    for (i, token) in tokens.iter().enumerate().take(15) {
        println!(
            "  {}: {:?} => '{}'",
            i,
            describe_token(token.kind),
            token.text
        );
    }

    println!();
    println!("=== Literal Parsing ===");
    demonstrate_literals();

    println!();
    println!("=== Comment and Whitespace Handling ===");
    demonstrate_trivia();

    println!();
    println!("=== Raw String Literals ===");
    demonstrate_raw_strings();

    println!();
    println!("=== Error Detection ===");
    demonstrate_error_handling();

    println!();
    println!("=== Advanced Token Kinds ===");
    demonstrate_advanced_tokens();

    println!();
    println!("=== Real Compiler Example ===");
    analyze_rust_code();
}

fn demonstrate_literals() {
    let literals = r##"
42              // decimal integer
0xFF            // hexadecimal
0o77            // octal
0b1010          // binary
3.14            // float
2.71e-10        // scientific notation
'a'             // character
'\n'            // escaped character
"hello"         // string
b"bytes"        // byte string
b'A'            // byte literal
r#"raw"#        // raw string
br#"raw bytes"# // raw byte string
"##;

    let mut lexer = Lexer::new(literals);
    let tokens = lexer.tokenize();

    println!("Literal tokens:");
    for token in tokens {
        if let TokenKind::Literal {
            kind,
            suffix_start: _,
        } = token.kind
        {
            match cook_lexer_literal(kind, &token.text, 0) {
                Ok(parsed) => {
                    println!("  {} => {:?}", token.text, parsed);
                }
                Err(e) => {
                    println!("  {} => Error: {:?}", token.text, e);
                }
            }
        }
    }
}

fn demonstrate_trivia() {
    let code = r#"
// This is a line comment
fn main() {
    /* This is a 
       block comment */
    println!("Hello"); // Another comment
}
"#;

    let mut lexer = Lexer::new(code);
    let all_tokens = lexer.tokenize_with_trivia();

    println!("All tokens including trivia:");
    let mut comment_count = 0;
    let mut whitespace_count = 0;

    for token in &all_tokens {
        match token.kind {
            TokenKind::LineComment => {
                comment_count += 1;
                println!("  Line comment: '{}'", token.text.trim());
            }
            TokenKind::BlockComment { terminated } => {
                comment_count += 1;
                println!(
                    "  Block comment (terminated: {}): '{}'",
                    terminated, token.text
                );
            }
            TokenKind::Whitespace => {
                whitespace_count += 1;
            }
            _ => {}
        }
    }

    println!();
    println!("Statistics:");
    println!("  Total tokens: {}", all_tokens.len());
    println!("  Comments: {}", comment_count);
    println!("  Whitespace tokens: {}", whitespace_count);

    // Now without trivia
    let mut lexer = Lexer::new(code);
    let code_tokens = lexer.tokenize();
    println!("  Code tokens only: {}", code_tokens.len());
}

fn demonstrate_raw_strings() {
    let raw_strings = r####"
r"simple raw"
r#"with "quotes""#
r##"with #"# hashes"##
r###"complex "##"### example"###
"####;

    let mut lexer = Lexer::new(raw_strings);
    let tokens = lexer.tokenize();

    println!("Raw string literals:");
    for token in tokens {
        if let TokenKind::Literal {
            kind:
                LiteralKind::RawStr {
                    n_hashes,
                    started,
                    terminated,
                },
            ..
        } = token.kind
        {
            if started && terminated {
                let parsed = cook_lexer_literal(
                    LiteralKind::RawStr {
                        n_hashes,
                        started,
                        terminated,
                    },
                    &token.text,
                    0,
                );
                if let Ok(ParsedLiteral::RawStr(content)) = parsed {
                    println!("  {} ({}# hashes) => \"{}\"", token.text, n_hashes, content);
                }
            }
        }
    }
}

fn demonstrate_error_handling() {
    let invalid_code = r#"
'unterminated char
"unterminated string
0x  // empty hex literal
3.14e  // empty exponent
/* unterminated comment
"#;

    match tokenize_and_validate(invalid_code) {
        Ok(_) => println!("No errors found (unexpected!)"),
        Err(errors) => {
            println!("Found {} validation errors:", errors.len());
            for error in errors {
                println!("  Position {}: {:?}", error.span.start, error.kind);
            }
        }
    }
}

fn demonstrate_advanced_tokens() {
    let advanced = r#"
'static
r#ident
$meta
@attribute
#[derive(Debug)]
Self::Associated
<T as Trait>::Type
|x| x + 1
..=
..
"#;

    let mut lexer = Lexer::new(advanced);
    let tokens = lexer.tokenize();

    println!("Advanced token types:");
    for token in tokens.iter().take(20) {
        match &token.kind {
            TokenKind::Lifetime { starts_with_number } => {
                println!(
                    "  Lifetime (starts with number: {}): {}",
                    starts_with_number, token.text
                );
            }
            TokenKind::RawIdent => {
                println!("  Raw identifier: {}", token.text);
            }
            TokenKind::Dollar => {
                println!("  Dollar sign (macro metavariable): {}", token.text);
            }
            TokenKind::At => {
                println!("  At sign (pattern binding): {}", token.text);
            }
            TokenKind::Pound => {
                println!("  Pound (attribute): {}", token.text);
            }
            _ => {
                if token.text.contains("..") {
                    println!("  Range operator: {}", token.text);
                }
            }
        }
    }
}

fn analyze_rust_code() {
    let compiler_code = r#"
/// A lexical analyzer for Rust-like languages
pub struct Lexer<'input> {
    input: &'input str,
    position: usize,
    tokens: Vec<Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            input,
            position: 0,
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) -> Result<&[Token], LexError> {
        while self.position < self.input.len() {
            match self.next_token() {
                Ok(token) => self.tokens.push(token),
                Err(e) => return Err(e),
            }
        }
        Ok(&self.tokens)
    }
}
"#;

    let mut lexer = Lexer::new(compiler_code);
    let tokens = lexer.tokenize_with_trivia();

    // Count different token categories
    let mut stats = TokenStats::default();

    for token in &tokens {
        match token.kind {
            TokenKind::Ident => {
                stats.identifiers += 1;
                match token.text.as_str() {
                    "pub" | "struct" | "impl" | "fn" | "let" | "match" | "while" => {
                        stats.keywords += 1
                    }
                    _ => {}
                }
            }
            TokenKind::Lifetime { .. } => stats.lifetimes += 1,
            TokenKind::Literal { .. } => stats.literals += 1,
            TokenKind::LineComment => {
                stats.comments += 1;
                // Check if it's a doc comment by examining the text
                if token.text.starts_with("///") || token.text.starts_with("//!") {
                    stats.doc_comments += 1;
                }
            }
            TokenKind::BlockComment { terminated: _ } => {
                stats.comments += 1;
                // Check if it's a doc comment by examining the text
                if token.text.starts_with("/**") || token.text.starts_with("/*!") {
                    stats.doc_comments += 1;
                }
            }
            TokenKind::OpenParen
            | TokenKind::CloseParen
            | TokenKind::OpenBrace
            | TokenKind::CloseBrace
            | TokenKind::OpenBracket
            | TokenKind::CloseBracket => stats.delimiters += 1,
            _ => {}
        }
    }

    println!("Token statistics for compiler code:");
    println!("  Total tokens: {}", tokens.len());
    println!(
        "  Identifiers: {} (including {} keywords)",
        stats.identifiers, stats.keywords
    );
    println!("  Lifetimes: {}", stats.lifetimes);
    println!("  Literals: {}", stats.literals);
    println!(
        "  Comments: {} (including {} doc comments)",
        stats.comments, stats.doc_comments
    );
    println!("  Delimiters: {}", stats.delimiters);

    // Show first few tokens with descriptions
    println!();
    println!("First 20 non-trivia tokens:");
    let mut count = 0;
    for token in &tokens {
        if !is_whitespace(token.kind) && !is_comment(token.kind) {
            println!("  {:15} => {}", token.text, describe_token(token.kind));
            count += 1;
            if count >= 20 {
                break;
            }
        }
    }
}

#[derive(Default)]
struct TokenStats {
    identifiers: usize,
    keywords: usize,
    lifetimes: usize,
    literals: usize,
    comments: usize,
    doc_comments: usize,
    delimiters: usize,
}
