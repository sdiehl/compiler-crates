# rustc_lexer

The rustc_lexer crate is the actual lexer used by the Rust compiler, extracted as a standalone library. Unlike traditional lexer generators, it provides a hand-written, highly optimized tokenizer specifically designed for the Rust language. This makes it invaluable for building Rust tooling, language servers, and compilers for Rust-like languages.

This lexer operates at the lowest level, producing raw tokens without any semantic understanding. It handles all of Rust's complex lexical features including raw strings, byte strings, numeric literals with various bases, and proper Unicode support. The lexer is designed for maximum performance and minimal allocation, making it suitable for incremental parsing scenarios.

## Basic Usage

The lexer provides a simple cursor-based API that produces one token at a time. Each token includes its kind and byte length in the source.

```rust
#![impl!("rustc_lexer/src/lib.rs", Lexer)]
```

This wrapper accumulates tokens into a vector for convenience. The lexer skips whitespace and comments by default, focusing on syntactically significant tokens.

## Token Kinds

The TokenKind enum covers all possible Rust tokens, from simple punctuation to complex literal forms. The lexer distinguishes between many subtle cases that are important for proper Rust parsing.

```rust
#![function!("rustc_lexer/src/lib.rs", describe_token)]
```

This function provides human-readable descriptions for each token kind, useful for error messages and debugging.

## Literal Processing  

Raw tokens need to be "cooked" to extract their actual values. The lexer identifies literal kinds but doesn't parse their contents, leaving that to a separate validation step.

```rust
#![function!("rustc_lexer/src/lib.rs", cook_lexer_literal)]
```

This function handles all of Rust's literal forms, including integer literals with different bases, floating-point numbers with scientific notation, character escapes, and various string literal types.

## Trivia Handling

Comments and whitespace (collectively called "trivia") can be preserved or discarded depending on the use case. Language servers need trivia for formatting, while parsers typically skip it.

This variant preserves all tokens including whitespace and comments, essential for tools that need to maintain source fidelity. The `tokenize_with_trivia` method on the Lexer struct returns all tokens without filtering.

This variant preserves all tokens including whitespace and comments, essential for tools that need to maintain source fidelity.

## Error Recovery

The lexer is designed for excellent error recovery, continuing to tokenize even when encountering invalid input. Unknown characters produce Unknown tokens rather than failing completely.

```rust
#![function!("rustc_lexer/src/lib.rs", tokenize_and_validate)]
```

This function combines tokenization with validation, collecting all errors while still producing a complete token stream. This approach enables IDEs to provide multiple error markers simultaneously.

## Raw Strings

Rust's raw string literals require special handling due to their configurable delimiters. The lexer tracks the number of pound signs and validates proper termination.

The lexer correctly handles arbitrarily nested pound signs in raw strings, making it possible to include any content without escaping. This is particularly useful for embedding other languages or test data in Rust code.

## Performance Characteristics

The rustc_lexer is highly optimized for the common case of valid Rust code. It uses table lookups for character classification and minimizes branching in hot paths. The lexer operates in linear time with respect to input size and performs no allocations during tokenization itself.

The cursor-based API allows for incremental lexing, where you can tokenize just a portion of the input or stop early based on some condition. This is crucial for responsive IDE experiences where files may be partially invalid during editing.

## Integration Patterns

For building a parser, wrap the lexer in a token stream that provides lookahead:

The lexer integrates naturally with parser combinators or hand-written recursive descent parsers. Its error recovery properties ensure the parser always has tokens to work with, even for invalid input.

For syntax highlighting, process tokens with trivia and map token kinds to color categories. The lexer's precise token classification enables accurate highlighting that matches rustc's interpretation.

## Best Practices

Cache the token stream when possible rather than re-lexing. While the lexer is fast, avoiding redundant work improves overall performance. For incremental scenarios, track which portions of the input have changed and re-lex only affected regions.

Validate literals in a separate pass rather than during lexing. This separation of concerns keeps the lexer simple and fast while allowing for better error messages during validation.

Handle both terminated and unterminated comments gracefully. IDEs need to provide reasonable behavior even when comments are unclosed, and the lexer's design supports this requirement.

The rustc_lexer provides a solid foundation for Rust language tooling. Its battle-tested implementation handles all the edge cases that make Rust lexing challenging, from raw identifiers to complex numeric literals. By using the same lexer as rustc, tools can ensure compatibility with the official Rust implementation.