# pest

Pest is a PEG (Parsing Expression Grammar) parser generator that uses a dedicated grammar syntax to generate parsers at compile time. Unlike parser combinators, pest separates grammar definition from parsing logic, enabling clear and maintainable parser specifications. The library excels at parsing complex languages with automatic whitespace handling, built-in error reporting, and elegant precedence climbing for expressions.

The core philosophy of pest centers on declarative grammar definitions that closely resemble formal language specifications. Grammars are written in separate `.pest` files using a clean, readable syntax that supports modifiers for different parsing behaviors. The pest_derive macro generates efficient parsers from these grammars at compile time.

## Grammar Fundamentals

```rust
#![source_file!("pest/src/grammar.pest")]
```

The grammar file demonstrates pest's PEG syntax with various rule types. Silent rules prefixed with underscore don't appear in the parse tree, simplifying AST construction. Atomic rules marked with `@` consume input without considering inner whitespace. Compound atomic rules using `$` capture the entire matched string as a single token.

## AST Construction

```rust
#![enum!("pest/src/lib.rs", Expr)]
```

The expression type represents the abstract syntax tree that parsers construct from pest's parse pairs. Each variant corresponds to grammatical constructs defined in the pest grammar.

```rust
#![function!("pest/src/lib.rs", GrammarParser::parse_expression)]
```

Expression parsing demonstrates the transformation from pest's generic parse tree to a typed AST. The PrattParser handles operator precedence through precedence climbing, supporting both left and right associative operators. The parser processes pairs recursively, matching rule names to construct appropriate AST nodes.

## Precedence Climbing

The Pratt parser configuration is built inline within the expression parser, defining operator precedence and associativity declaratively. Left associative operators like addition and multiplication are specified with `infix`, while right associative operators like exponentiation use `infix` with right associativity. Prefix operators for unary expressions integrate seamlessly with the precedence system.

```rust
#![function!("pest/src/lib.rs", GrammarParser::parse_calculation)]
```

The calculator parser evaluates expressions directly during parsing, demonstrating how precedence climbing produces correct results. Right associative exponentiation evaluates from right to left, while other operators evaluate left to right. This approach combines parsing and evaluation for simple expression languages.

## JSON Parsing

```rust
#![enum!("pest/src/lib.rs", JsonValue)]
```

JSON parsing showcases pest's handling of recursive data structures. The grammar defines objects, arrays, strings, numbers, and literals with appropriate nesting rules.

```rust
#![function!("pest/src/lib.rs", GrammarParser::parse_json)]
```

The JSON parser transforms pest pairs into a typed representation. Match expressions on rule types drive the recursive construction of nested structures. String escape sequences and number parsing are handled during AST construction rather than in the grammar.

## Programming Language Constructs

```rust
#![enum!("pest/src/lib.rs", Statement)]
```

Statement types demonstrate parsing of control flow and program structure. The grammar supports if statements, while loops, function definitions, and variable declarations.

```rust
#![function!("pest/src/lib.rs", GrammarParser::parse_program)]
```

Program parsing builds complex ASTs from multiple statement types. The recursive nature of the parser handles nested blocks and control structures. Pattern matching on rule names provides clear correspondence between grammar and implementation.

## Token Stream Parsing

```rust
#![enum!("pest/src/lib.rs", Token)]
```

Token extraction demonstrates pest's ability to function as a lexer. The grammar identifies different token types while preserving their textual representation.

```rust
#![function!("pest/src/lib.rs", GrammarParser::parse_tokens)]
```

Token stream parsing extracts lexical elements from source text. Each token preserves its span information for error reporting and source mapping. The approach separates lexical analysis from syntactic parsing when needed.

## Error Handling

```rust
#![enum!("pest/src/lib.rs", ParseError)]
```

Custom error types provide context-specific error messages beyond pest's built-in reporting. The error variants correspond to different failure modes in parsing and semantic analysis.

```rust
#![function!("pest/src/lib.rs", GrammarParser::debug_parse)]
```

Debug parsing enriches pest's error messages with domain-specific information. The function wraps pest errors with additional context about what was being parsed. Line and column information from pest integrates with custom error formatting.

## Utility Functions

```rust
#![function!("pest/src/lib.rs", GrammarParser::extract_identifiers)]
```

Helper functions simplify common parsing patterns. Identifier extraction demonstrates traversing the parse tree to collect specific elements. The visitor pattern works well with pest's pair structure for gathering information.

Debug utilities help understand pest's parse tree structure during development. The function recursively prints the tree with indentation showing nesting levels. Rule names and captured text provide insight into how the grammar matched input.

## Grammar Design Patterns

Pest grammars benefit from consistent structure and naming conventions. Use snake_case for rule names and UPPER_CASE for token constants. Group related rules together with comments explaining their purpose. Silent rules with underscore prefixes hide implementation details from the parse tree.

Whitespace handling deserves special attention in grammar design. The built-in WHITESPACE rule automatically skips whitespace between tokens. Atomic rules disable automatic whitespace handling when exact matching is required. Use push and pop operations for significant indentation in languages like Python.

Comments can be handled uniformly through the COMMENT rule. Single-line and multi-line comment patterns integrate naturally with automatic skipping. This approach keeps the main grammar rules clean and focused on language structure.

## Performance Optimization

Pest generates efficient parsers through compile-time code generation. The generated code uses backtracking only when necessary, preferring deterministic parsing where possible. Memoization of rule results prevents redundant parsing of the same input.

Rule ordering impacts performance in choice expressions. Place more common alternatives first to reduce backtracking. Use atomic rules to prevent unnecessary whitespace checking in tight loops. Consider breaking complex rules into smaller components for better caching.

The precedence climbing algorithm provides optimal performance for expression parsing. Unlike naive recursive descent, it avoids deep recursion for left-associative operators. The algorithm handles arbitrary precedence levels efficiently without grammar transformations.

## Integration Patterns

Pest integrates well with other Rust compiler infrastructure. Parse results can be converted to spans for error reporting libraries like codespan or ariadne. The AST types can implement serde traits for serialization or visitor patterns for analysis passes.

Incremental parsing can be implemented by caching parse results for unchanged input sections. The stateless nature of pest parsers enables parallel parsing of independent input chunks. Custom pair processing can extract only needed information without full AST construction.

Testing pest grammars requires attention to both positive and negative cases. Use pest's built-in testing syntax in grammar files for quick validation. Integration tests should verify AST construction and error handling. Property-based testing can validate grammar properties like precedence and associativity.

## Best Practices

Keep grammars readable and maintainable by avoiding overly complex rules. Break down complicated patterns into named sub-rules that document their purpose. Use meaningful rule names that correspond to language concepts rather than implementation details.

Version control grammar files alongside implementation code. Document grammar changes and their rationale in commit messages. Consider grammar compatibility when evolving languages to avoid breaking existing code.

Profile parser performance on representative input to identify bottlenecks. Complex backtracking patterns or excessive rule nesting can impact performance. Use pest's built-in debugging features to understand parsing behavior on problematic input.

Handle errors gracefully with informative messages. Pest's automatic error reporting provides good default messages, but custom errors can add domain-specific context. Consider recovery strategies for IDE integration where partial results are valuable.
