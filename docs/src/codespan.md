# codespan

Codespan provides fundamental span tracking and position management infrastructure for compiler diagnostics and source mapping. The crate offers precise byte-level position tracking with efficient conversion to line and column information, making it ideal for error reporting and source code analysis. Unlike higher-level diagnostic libraries, codespan focuses on the core span arithmetic and position calculations that underpin accurate source location tracking.

The library centers around immutable position types that represent byte offsets in source text. These types support arithmetic operations for calculating ranges and distances while maintaining type safety. The span abstraction represents a contiguous range of source text, enabling precise tracking of token locations, expression boundaries, and error positions throughout compilation.

## Core Position Types

```rust
#![struct!("codespan/src/lib.rs", SourceFile)]
```

The SourceFile structure maintains the source text along with precomputed line start positions for efficient line and column calculation. The line starts vector enables binary search for position lookups, providing O(log n) complexity for location queries.

```rust
#![function!("codespan/src/lib.rs", SourceFile::new)]
```

File construction scans the input once to identify line boundaries, building an index for subsequent position queries. This upfront computation trades initialization time for faster repeated lookups during compilation.

```rust
#![function!("codespan/src/lib.rs", SourceFile::line_index)]
```

Line index calculation uses binary search on the precomputed line starts. When the search finds an exact match, that line contains the position. Otherwise, the position falls within the preceding line.

```rust
#![function!("codespan/src/lib.rs", SourceFile::column_index)]
```

Column calculation first determines the line, then computes the byte offset from the line start. This approach handles variable-width characters correctly by operating on byte positions rather than character counts.

## Span Management

```rust
#![struct!("codespan/src/lib.rs", SpanManager)]
```

The SpanManager coordinates multiple source files in a compilation unit. It provides centralized file registration and lookup while maintaining consistent file identifiers across the compilation pipeline.

```rust
#![function!("codespan/src/lib.rs", SpanManager::add_file)]
```

File registration assigns sequential identifiers and maintains a name-based lookup table. This design supports both positional access for span resolution and name-based queries for import resolution.

```rust
#![function!("codespan/src/lib.rs", SpanManager::merge_spans)]
```

Span merging combines multiple spans into their encompassing range. This operation proves essential for error reporting when an error spans multiple tokens or when synthesizing spans for derived expressions.

## Token Representation

```rust
#![struct!("codespan/src/lib.rs", Token)]
```

Tokens carry their span information throughout parsing and analysis. The generic type parameter allows reuse across different token representations while maintaining consistent span tracking.

```rust
#![enum!("codespan/src/lib.rs", TokenKind)]
```

The token enumeration demonstrates typical language constructs that benefit from span tracking. Each variant can be associated with its source location for precise error reporting.

## Lexical Analysis

```rust
#![struct!("codespan/src/lib.rs", Lexer)]
```

The lexer maintains position state while scanning input text. It tracks byte positions rather than character indices to handle UTF-8 text correctly.

```rust
#![function!("codespan/src/lib.rs", Lexer::tokenize)]
```

Tokenization produces a stream of tokens with associated spans. Each token records its start and end positions, enabling accurate source mapping for error messages and debugging information.

```rust
#![function!("codespan/src/lib.rs", Lexer::scan_token)]
```

Token scanning dispatches on the first character to identify token types. The function advances through the input while tracking byte positions for span construction.

```rust
#![function!("codespan/src/lib.rs", Lexer::scan_string)]
```

String scanning demonstrates escape sequence handling while maintaining accurate span information. The lexer tracks positions through escaped characters to ensure spans accurately reflect source locations.

## Position Arithmetic

```rust
#![function!("codespan/src/lib.rs", demonstrate_span_arithmetic)]
```

Span arithmetic operations enable position calculations throughout the compiler. ByteIndex represents absolute positions while ByteOffset represents relative distances, maintaining type safety in position calculations.

```rust
#![function!("codespan/src/lib.rs", demonstrate_line_offsets)]
```

Line offset calculations mirror byte-level operations at the line granularity. These operations support navigation between error locations and related positions in diagnostic output.

## UTF-8 Support

```rust
#![function!("codespan/src/lib.rs", track_utf8_positions)]
```

UTF-8 position tracking correctly handles variable-width characters. The function accumulates byte offsets based on actual character encoding lengths rather than assuming fixed-width characters.

## Location Display

```rust
#![struct!("codespan/src/lib.rs", Location)]
```

The Location structure provides human-readable position information. Line and column indices are zero-based internally but typically displayed with one-based numbering for user interfaces.

## Testing Strategies

The test suite validates core positioning operations across various text encodings. UTF-8 handling receives particular attention given the complexity of multi-byte character sequences.

Binary search performance in line lookup benefits from representative test data. Large files with many lines exercise the logarithmic lookup behavior that makes the approach scalable.

Span merging tests verify the commutativity and associativity properties that simplify span combination logic. These algebraic properties enable optimization passes to freely reorganize span calculations.

## Integration Patterns

Codespan integrates naturally with parser combinators and parser generators. Parser libraries can construct spans from their internal position tracking, while codespan provides the arithmetic operations for span manipulation.

Error reporting libraries like codespan-reporting build on these primitives to provide rich diagnostic output. The separation of concerns keeps span tracking focused and reusable across different diagnostic frameworks.

Incremental compilation benefits from stable span representations. Source positions remain valid across incremental updates when unchanged regions retain their byte offsets.

## Performance Considerations

Line start precomputation trades memory for lookup speed. For typical source files, the line index overhead remains negligible compared to the source text itself.

Binary search for line lookup provides consistent performance regardless of file size. This scalability matters for generated code or concatenated source files that may contain thousands of lines.

Span creation involves only copying two integers, making it efficient to track spans pervasively. The immutable nature of spans enables sharing without synchronization overhead.

## Best Practices

Maintain span information throughout compilation rather than reconstructing it when needed. Early span loss complicates error reporting and prevents accurate source mapping.

Use typed position wrappers rather than raw integers to prevent unit confusion. The distinction between ByteIndex and ByteOffset catches common arithmetic errors at compile time.

Prefer byte positions over character positions for internal representation. Byte positions provide unambiguous locations in UTF-8 text while character positions require encoding assumptions.

Design token types to be lightweight since they carry span information. Large token payloads amplify memory usage when every token includes span data.

Codespan's focused approach to span tracking provides a solid foundation for compiler diagnostics. The crate's emphasis on correctness, particularly regarding UTF-8 handling, makes it suitable for production language implementations where accurate position tracking directly impacts developer experience.