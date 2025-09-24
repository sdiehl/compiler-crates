# chumsky

Chumsky is a parser combinator library that emphasizes error recovery, performance, and ease of use. Unlike traditional parser generators, chumsky builds parsers from small, composable functions that can be combined to parse complex grammars. The library excels at providing detailed error messages and recovering from parse errors to continue processing malformed input.

Parser combinators in chumsky follow a functional programming style where parsers are values that can be composed using combinator functions. Each parser is a function that consumes input and produces either a parsed value or an error. The library provides extensive built-in combinators for common patterns like repetition, choice, and sequencing.

## Core Parser Types

```rust
#![enum!("chumsky/src/lib.rs", Expr)]
```

The expression type represents the abstract syntax tree nodes that parsers produce. Chumsky parsers transform character streams into structured data like this AST.

```rust
#![enum!("chumsky/src/lib.rs", BinOp)]
```

Binary operators demonstrate how parsers handle operator precedence and associativity through careful combinator composition.

## Building Expression Parsers

```rust
#![function!("chumsky/src/lib.rs", expr_parser)]
```

The expression parser showcases several key chumsky features. The `recursive` combinator enables parsing recursive structures like nested expressions. The `choice` combinator tries multiple alternatives until one succeeds. The `foldl` combinator builds left-associative binary operations by folding a list of operators and operands.

Operator precedence emerges naturally from parser structure. Parsers for higher-precedence operators like multiplication appear lower in the combinator chain, ensuring they bind more tightly than addition or subtraction. The `then` combinator sequences parsers, while `map` transforms parsed values into AST nodes.

## Lexical Analysis

```rust
#![enum!("chumsky/src/lib.rs", Token)]
```

While chumsky can parse character streams directly, separate lexical analysis often improves performance and error messages for complex languages.

```rust
#![function!("chumsky/src/lib.rs", lexer)]
```

The lexer demonstrates span tracking, which records the source location of each token. The `map_with_span` combinator attaches location information to parsed values, enabling precise error reporting. Keywords are distinguished from identifiers during lexing rather than parsing, simplifying the grammar.

## Error Recovery

```rust
#![function!("chumsky/src/lib.rs", robust_parser)]
```

Error recovery allows parsers to continue processing after encountering errors, producing partial results and multiple error messages. The `recover_with` combinator specifies recovery strategies for specific error conditions. The `nested_delimiters` recovery strategy handles mismatched parentheses by searching for the appropriate closing delimiter.

Recovery strategies help development tools provide better user experiences. IDEs can show multiple syntax errors simultaneously, and compilers can report more problems in a single run. The `separated_by` combinator with `allow_trailing` handles comma-separated lists gracefully, even with trailing commas.

## Custom Combinators

```rust
#![function!("chumsky/src/lib.rs", binary_op_parser)]
```

Custom combinators encapsulate common parsing patterns for reuse across different parts of a grammar. This binary operator parser handles any set of operators at the same precedence level, building left-associative expressions. The generic implementation works with any operator type and expression parser.

Creating domain-specific combinators improves grammar readability and reduces duplication. Common patterns in a language can be abstracted into reusable components that compose naturally with built-in combinators.

## Validation and Semantic Analysis

```rust
#![function!("chumsky/src/lib.rs", validated_parser)]
```

The `validate` combinator performs semantic checks during parsing, emitting errors for invalid constructs while continuing to parse. This enables reporting both syntactic and semantic errors in a single pass. Validation can check numeric ranges, identifier validity, or any other semantic constraint.

Combining parsing and validation reduces the number of passes over the input and provides better error messages by retaining parse context. The error emission mechanism allows multiple errors from a single validation, supporting comprehensive error reporting.

## Performance Considerations

Chumsky parsers achieve good performance through several optimizations. The library uses zero-copy parsing where possible, avoiding string allocation for tokens and identifiers. Parsers are compiled to efficient state machines that minimize backtracking.

Choice combinators try alternatives in order, so placing common cases first improves performance. The `or` combinator creates more efficient parsers than `choice` when only two alternatives exist. Memoization can be added to recursive parsers to avoid reparsing the same input multiple times.

## Integration Patterns

Chumsky integrates well with other compiler infrastructure. The span information works with error reporting libraries like ariadne or codespan-reporting to display beautiful error messages. AST nodes can implement visitor patterns or be processed by subsequent compiler passes.

The streaming API supports parsing large files without loading them entirely into memory. Incremental parsing can be implemented by caching parse results for unchanged portions of input. The modular parser design allows testing individual components in isolation.

## Best Practices

Structure parsers hierarchically, with each level handling one precedence level or syntactic category. Use meaningful names for intermediate parsers to improve readability. Keep individual parsers focused on a single responsibility.

Test parsers thoroughly with both valid and invalid input. Error recovery strategies should be tested to ensure they produce reasonable partial results. Use property-based testing to verify parser properties like round-tripping through pretty-printing.

Profile parser performance on realistic input to identify bottlenecks. Complex lookahead or backtracking can dramatically impact performance. Consider using a separate lexer for languages with complex tokenization rules.

Document grammar ambiguities and their resolution strategies. Explain why certain parser structures were chosen, especially for complex precedence hierarchies. Provide examples of valid and invalid syntax to clarify language rules.
