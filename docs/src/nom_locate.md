# nom_locate

The `nom_locate` crate extends nom's parser combinators with precise source location tracking. While nom excels at building fast, composable parsers, it doesn't inherently track where in the source text each parse occurred. For compiler construction, this location information is crucial - every error message, warning, and diagnostic needs to point users to the exact position in their source code. nom_locate solves this by wrapping input slices with location metadata that flows through the parsing process.

Location tracking enables compilers to provide helpful error messages that show not just what went wrong, but exactly where. It also supports advanced IDE features like go-to-definition, hover information, and refactoring tools that need to map between source text and AST nodes. The crate integrates seamlessly with nom's existing combinators while adding minimal overhead to parsing performance.

## Core Types

The foundation of nom_locate is the `LocatedSpan` type, typically aliased for convenience:

```rust
type Span<'a> = LocatedSpan<&'a str>;
```

```rust
#![struct!("nom-locate/src/lib.rs", Location)]
```

```rust
#![struct!("nom-locate/src/lib.rs", SourceRange)]
```

These types provide line, column, and byte offset information for any parsed element.

## Creating a Located Parser

Transform a nom parser to track locations by wrapping the input:

```rust
#![struct!("nom-locate/src/lib.rs", Parser)]
```

The parser methods work with `Span` instead of `&str`, automatically tracking positions as parsing proceeds.

## Expression Parsing with Locations

Building an expression parser that preserves source locations:

```rust
#![enum!("nom-locate/src/lib.rs", Expr)]
```

```rust
#![struct!("nom-locate/src/lib.rs", Spanned)]
```

Every AST node is wrapped with `Spanned` to preserve its source location alongside the parsed data.

## Extracting Position Information

Convert nom_locate's position data into user-friendly line and column numbers:

```rust
#![function!("nom-locate/src/lib.rs", Parser::get_position_info)]
```

This provides the exact line and column for error reporting and IDE features.

## Error Reporting with Context

Generate helpful error messages with source context:

```rust
#![function!("nom-locate/src/lib.rs", Parser::get_line_content)]
```

This creates error displays showing the problematic line with a pointer to the exact error location.

## Tokenizer with Location Tracking

Building a located lexer that preserves position information for each token:

```rust
#![struct!("nom-locate/src/lib.rs", LocatedLexer)]
```

```rust
#![struct!("nom-locate/src/lib.rs", LocatedToken)]
```

```rust
#![enum!("nom-locate/src/lib.rs", TokenKind)]
```

Each token knows exactly where it appeared in the source text, enabling precise error messages even from later compilation phases.

## Binary Expression Parsing

Handling operator precedence while maintaining location information:

```rust
#![enum!("nom-locate/src/lib.rs", BinaryOp)]
```

The parser correctly handles precedence while tracking the span of entire expressions and their sub-components.

## Best Practices

Preserve spans throughout AST construction. Don't extract the inner value and discard location information until absolutely necessary.

Use type aliases to make span types more readable. `type Span<'a> = LocatedSpan<&'a str>` is clearer than using the full type everywhere.

Create wrapper functions for common patterns. Helper functions that handle span extraction and position calculation reduce boilerplate.

Test location accuracy. Include tests that verify not just parse results but also that locations are correctly preserved.

Design AST nodes to include location information from the start. Retrofitting location tracking is much harder than including it in the initial design.
