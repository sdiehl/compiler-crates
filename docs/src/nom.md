# nom

Nom is a parser combinator library focused on binary formats and text protocols, emphasizing zero-copy parsing and streaming capabilities. The library uses a functional programming approach where small parsers combine into larger ones through combinator functions. Nom excels at parsing network protocols, file formats, and configuration languages with excellent performance characteristics.

The core abstraction in nom is the `IResult` type, which represents the outcome of a parser. Every parser consumes input and produces either a successful parse with remaining input or an error. This design enables parsers to chain naturally, with each parser consuming part of the input and passing the remainder to the next parser.

## Core Types and Parsers

```rust
#![enum!("nom/src/lib.rs", Expr)]
```

The expression type demonstrates a typical AST that nom parsers produce. Each variant represents a different syntactic construct that the parser recognizes.

## Number Parsing

```rust
#![function!("nom/src/lib.rs", float)]
```

The float parser showcases nom's approach to parsing numeric values. The `recognize` combinator captures the matched input as a string slice, while `map_res` applies a fallible transformation. This pattern avoids allocation by working directly with input slices.

```rust
#![function!("nom/src/lib.rs", integer)]
```

Integer parsing follows a similar pattern but handles signed integers. The `pair` combinator sequences two parsers, and `opt` makes a parser optional, enabling parsing of both positive and negative numbers.

## String and Identifier Parsing

```rust
#![function!("nom/src/lib.rs", string_literal)]
```

String literal parsing demonstrates nom's handling of escape sequences. The `escaped` combinator recognizes escaped characters within strings, supporting common escape sequences like newlines and quotes. The `delimited` combinator extracts content between delimiters.

```rust
#![function!("nom/src/lib.rs", identifier)]
```

Identifier parsing shows how to build parsers for programming language tokens. The `recognize` combinator returns the matched input slice rather than the parsed components, avoiding string allocation. The `alt` combinator tries multiple alternatives until one succeeds.

## Expression Parsing

```rust
#![function!("nom/src/lib.rs", expression)]
```

Expression parsing demonstrates operator precedence through parser layering. The `fold_many0` combinator implements left-associative binary operators by folding a sequence of operations. Higher precedence operations like multiplication are parsed in the `term` function, called from within expression parsing.

The separation of `term` and `expression` functions creates the precedence hierarchy. Terms handle multiplication and division, while expressions handle addition and subtraction. This structure ensures correct operator precedence without explicit precedence declarations.

## Function Calls and Arrays

```rust
#![function!("nom/src/lib.rs", function_call)]
```

Function call parsing combines several nom features. The `tuple` combinator sequences multiple parsers, capturing all results. The `separated_list0` combinator handles comma-separated argument lists, a common pattern in programming languages.

```rust
#![function!("nom/src/lib.rs", array)]
```

Array parsing uses similar techniques but with different delimiters. The `ws` helper function handles whitespace around tokens, a critical aspect of parsing human-readable formats.

## Configuration File Parsing

```rust
#![struct!("nom/src/lib.rs", Config)]
```

```rust
#![enum!("nom/src/lib.rs", Value)]
```

Configuration parsing demonstrates nom's suitability for structured data formats. The types represent a typical configuration file structure with sections and key-value pairs.

```rust
#![function!("nom/src/lib.rs", parse_config)]
```

The configuration parser builds up from smaller parsers for values, entries, and sections. Each parser focuses on one aspect of the format, combining through nom's compositional approach. The `many0` combinator parses zero or more occurrences, building collections incrementally.

## Error Handling

```rust
#![function!("nom/src/lib.rs", parse_with_context)]
```

Context-aware parsing improves error messages by annotating parsers with descriptive labels. The `context` combinator wraps parsers with error context, while `cut` prevents backtracking after partial matches. This combination provides precise error messages indicating exactly where parsing failed.

The `VerboseError` type collects detailed error information including the error location and a trace of attempted parses. This information helps developers understand why parsing failed and where in the grammar the error occurred.

## Streaming and Binary Parsing

```rust
#![function!("nom/src/lib.rs", streaming_parser)]
```

Streaming parsing handles input that may not be completely available. The parser processes available data and indicates how much input was consumed. This approach works well for network protocols and large files that cannot fit in memory.

```rust
#![function!("nom/src/lib.rs", parse_binary_header)]
```

Binary format parsing showcases nom's byte-level parsing capabilities. The library provides parsers for various integer encodings, network byte order, and fixed-size data. The `take` combinator extracts a specific number of bytes, while endian-specific parsers handle byte order conversions.

## Performance Optimization

Nom achieves excellent performance through zero-copy parsing. Parsers work directly with input slices, avoiding string allocation until necessary. The `recognize` combinator returns matched input slices, and parsers can pass ownership of subslices rather than copying data.

Careful combinator choice impacts performance. The `alt` combinator tries alternatives sequentially, so placing common cases first reduces average parsing time. The `many0` and `many1` combinators can be replaced with `fold_many0` and `fold_many1` to avoid intermediate vector allocation.

Nom's macros generate specialized code for each parser combination, eliminating function call overhead. The generated code often compiles to efficient machine code comparable to hand-written parsers.

## Integration Patterns

Nom parsers integrate well with other Rust libraries. The `&str` and `&[u8]` input types work with standard library types, while the `IResult` type integrates with error handling libraries. Parsed ASTs can be processed by subsequent compiler passes or serialized to other formats.

For incremental parsing, nom parsers can save state between invocations. The remaining input from one parse becomes the starting point for the next, enabling parsing of streaming data or interactive input.

Custom input types allow parsing from non-standard sources. Implementing nom's input traits enables parsing from rope data structures, memory-mapped files, or network streams.

## Best Practices

Structure parsers hierarchically with clear separation of concerns. Each parser should handle one grammatical construct, making the grammar evident from the code structure. Use descriptive names that match the grammar terminology.

Test parsers extensively with both valid and invalid input. Property-based testing verifies parser properties like consuming all valid input or rejecting invalid constructs. Fuzzing finds edge cases in parser implementations.

Profile parsers on representative input to identify performance bottlenecks. Complex alternatives or excessive backtracking impact performance. Consider using `peek` to look ahead without consuming input when making parsing decisions.

Handle errors gracefully with appropriate error types. The `VerboseError` type aids development, while custom error types provide better user experience. Use `context` and `cut` to improve error messages.

Document the grammar alongside the parser implementation. Comments should explain the grammatical constructs being parsed and any deviations from standard grammar notation. Examples of valid input clarify the parser's behavior.

The combination of nom's performance, composability, and support for various input types makes it ideal for parsing network protocols, file formats, and domain-specific languages. The functional approach encourages modular parser design that scales from simple expressions to complex languages.