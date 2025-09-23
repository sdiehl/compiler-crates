# logos

The logos crate provides a fast, derive-based lexer generator for Rust. Unlike traditional lexer generators that produce separate source files, logos integrates directly into your Rust code through derive macros. It generates highly optimized, table-driven lexers that can process millions of tokens per second, making it ideal for production compilers and language servers.

Logos excels at tokenizing programming languages with its declarative syntax for defining token patterns. The generated lexers use efficient DFA-based matching with automatic longest-match semantics. The crate handles common compiler requirements like source location tracking, error recovery, and stateful lexing for context-sensitive tokens like string literals or nested comments.

## Basic Token Definition

Token types in logos are defined as enums with the `#[derive(Logos)]` attribute. Each variant represents a different token type, annotated with patterns that match the token.

```rust
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\f]+")] // Skip whitespace except newlines
pub enum Token {
    // Keywords
    #[token("fn")]
    Function,
    #[token("let")]
    Let,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    
    // Identifiers and literals
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(Option<i64>),
    
    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
}
```

The `#[token]` attribute matches exact strings, while `#[regex]` matches regular expressions. The skip directive tells logos to automatically skip whitespace between tokens. Tokens can capture data by providing a closure that processes the matched text.

## Using the Lexer

Logos generates an iterator-based lexer that processes input incrementally. The lexer provides access to the matched token, its span in the source, and the matched text.

```rust
#![function!("logos/src/lib.rs", tokenize)]
```

This function demonstrates basic tokenization, collecting all tokens with their source spans. The spans are byte offsets that can be used to extract the original text or generate error messages with precise locations.

## Error Handling

Real compilers need robust error handling for invalid input. Logos returns `Result<Token, ()>` for each token, allowing graceful handling of lexical errors.

```rust
#![function!("logos/src/lib.rs", tokenize_with_errors)]
```

This approach separates valid tokens from error spans, enabling the compiler to continue processing after encountering invalid characters. Error spans can be used to generate diagnostics showing exactly where the problem occurred.

## Source Location Tracking

Compilers need to map byte offsets to human-readable line and column numbers for error reporting. The SourceTracker maintains this mapping efficiently.

```rust
#![struct!("logos/src/lib.rs", SourceTracker)]
```

The SourceTracker builds an index of line start positions for efficient location queries, scanning the input once at initialization to find all newline positions.

The tracker pre-computes line boundaries for O(log n) location lookups. This is crucial for language servers that need to convert between byte offsets and editor positions frequently.

## Token Streams

For parsing, it's often useful to wrap the lexer in a stream that supports peeking at the next token without consuming it.

```rust
#![struct!("logos/src/lib.rs", TokenStream)]
```

The peek_token method allows the parser to look at the next token without consuming it, enabling predictive parsing algorithms to make decisions based on lookahead.

The token stream maintains a one-token lookahead buffer, essential for predictive parsing techniques like recursive descent.

## Advanced Patterns

Logos supports complex token patterns including comments, escape sequences in strings, and numeric literals with different bases. The derive macro generates efficient matching code for all these patterns.

Comments can be handled by marking them with `logos::skip` to automatically discard them, or by capturing them as tokens if needed for documentation processing. String literals can use regex patterns to handle escape sequences, while numeric literals can parse different bases and formats.

## Stateful Lexing

Some languages require context-sensitive lexing, such as Python's significant indentation. Logos supports stateful lexing through the extras system.

```rust
#![struct!("logos/src/lib.rs", IndentationTracker)]
```

The extras field is accessible in token callbacks, allowing the lexer to maintain state between tokens. This enables proper handling of indent/dedent tokens in indentation-sensitive languages.

## Performance Characteristics

Logos generates table-driven DFAs that process input in linear time with minimal branching. The generated code uses Rust's zero-cost abstractions effectively, with performance comparable to hand-written lexers. Benchmarks show throughput of 50-200 MB/s on modern hardware, depending on token complexity.

The lexer allocates only for captured token data like identifiers and string literals. Token matching itself is allocation-free, making logos suitable for incremental lexing in language servers where performance is critical.

## Best Practices

Design your token enum to minimize allocations. Use zero-sized variants for keywords and operators, capturing data only when necessary. Order patterns from most specific to least specific to ensure correct matching precedence.

Keep regular expressions simple and avoid backtracking patterns. Logos compiles regexes to DFAs at compile time, so complex patterns increase compilation time and generated code size. For truly complex patterns like string interpolation, consider using a two-phase approach with a simpler lexer followed by specialized parsing.

Handle whitespace and comments consistently. Use the skip directive for insignificant whitespace, but consider preserving comments if you need them for documentation generation or code formatting. The lexer can emit comment tokens that the parser can then choose to ignore.

Logos integrates well with parser combinators and hand-written recursive descent parsers. Its iterator interface and error handling make it a natural fit for Rust's parsing ecosystem, providing a solid foundation for building efficient language processors.