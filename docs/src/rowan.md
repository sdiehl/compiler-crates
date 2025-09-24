# rowan

The `rowan` crate provides a foundation for building lossless syntax trees that preserve all source text including whitespace and comments. This architecture forms the basis of rust-analyzer and enables incremental reparsing, precise error recovery, and full-fidelity source transformations. Unlike traditional abstract syntax trees that discard formatting information, rowan maintains a complete representation of the input text while providing efficient tree traversal and manipulation operations.

The core innovation of rowan lies in its separation of untyped green trees from typed red trees. Green trees are immutable, shareable nodes that store the actual data, while red trees provide a typed API with parent pointers and absolute positions. This dual structure enables both memory efficiency through structural sharing and ergonomic traversal through the red tree API. The design supports incremental updates by allowing subtrees to be reused when portions of the source remain unchanged.

## Basic Usage

Rowan operates through a language definition that maps syntax kinds to tree nodes:

```rust
#![function!("rowan/src/lib.rs", parse_expression)]
```

The parse_expression function demonstrates the complete pipeline from source text to syntax tree. The tokenizer produces a stream of tokens with their kinds and positions, the parser builds a green tree using these tokens, and finally the syntax tree builder creates the typed red tree for traversal.

## Language Definition

Every rowan-based parser requires a language definition that specifies the syntax kinds and their conversions:

```rust
#![enum!("rowan/src/lib.rs", SyntaxKind)]
```

```rust
#![enum!("rowan/src/lib.rs", Lang)]
```

The SyntaxKind enumeration defines all possible node and token types in the language. Each variant represents either a terminal token like an identifier or operator, or a non-terminal node like an expression or statement. The Lang type implements the Language trait, providing the bridge between rowan's generic infrastructure and the specific syntax kinds.

## Green Tree Construction

The parser builds green trees using the GreenNodeBuilder API:

```rust
#![struct!("rowan/src/lib.rs", Parser)]
```

```rust
#![function!("rowan/src/lib.rs", parse_expression)]
```

The parser maintains a builder that constructs the green tree incrementally. The start_node and finish_node methods create hierarchical structure, while the token method adds leaf nodes. This approach allows the parser to build the tree in a single pass without backtracking or tree rewriting.

## Expression Parsing

The parser implements recursive descent with operator precedence for expressions:

```rust
#![impl!("rowan/src/lib.rs", Parser)]
```

Binary expression parsing uses precedence climbing to handle operator priorities correctly. The method recursively parses higher-precedence expressions on the right side, building a left-associative tree structure. The checkpoint mechanism allows the parser to reorganize nodes during parsing without rebuilding the entire subtree.

## Statement Parsing

Statement parsing demonstrates the handling of control flow constructs. The parser handles nested structures like if-else chains and function definitions by recursively invoking the appropriate parsing methods. Each statement type creates its own node in the syntax tree, preserving the complete structure including keywords, delimiters, and nested blocks.

## Tokenization

The tokenizer converts source text into a stream of typed tokens:

```rust
#![function!("rowan/src/lib.rs", tokenize)]
```

Tokenization tracks both the token content and its position in the source text using TextSize. This position information enables accurate error reporting and supports incremental reparsing by identifying which tokens have changed. The tokenizer handles multi-character tokens like comments and strings by consuming characters until reaching a delimiter.

## Incremental Reparsing

Rowan supports efficient incremental updates through text edits:

```rust
#![struct!("rowan/src/lib.rs", IncrementalReparser)]
```

```rust
#![impl!("rowan/src/lib.rs", IncrementalReparser)]
```

The incremental reparser tracks edits to the source text and efficiently rebuilds only the affected portions of the syntax tree. This capability is crucial for IDE scenarios where the source changes frequently and full reparsing would be prohibitively expensive. The reparser identifies unchanged subtrees that can be reused from the previous parse.

## AST Layer

The AST layer provides a typed interface over the syntax tree:

```rust
#![struct!("rowan/src/lib.rs", AstNode)]
```

```rust
#![trait!("rowan/src/lib.rs", AstToken)]
```

AST nodes wrap syntax nodes with type-safe accessors for their children and properties. The cast method performs runtime type checking to ensure the syntax node has the expected kind. This layer provides the ergonomic API that language servers and other tools use to analyze and transform code.

## Tree Traversal

Rowan provides utilities for navigating and searching the syntax tree:

```rust
#![function!("rowan/src/lib.rs", walk_tree)]
```

```rust
#![function!("rowan/src/lib.rs", find_node_at_offset)]
```

Tree traversal functions enable common IDE operations like finding the syntax node at a cursor position or collecting all nodes of a specific type. The find_node_at_offset function is particularly useful for implementing hover information and go-to-definition features in language servers.

## Best Practices

Design your syntax kinds hierarchy to balance granularity with usability. Too few kinds make the tree difficult to analyze, while too many create unnecessary complexity. Group related tokens into categories like operators or keywords when they behave similarly in the grammar.

Implement error recovery in the parser to produce valid trees even for incorrect input. Skip unexpected tokens rather than failing completely, and use error nodes to mark problematic regions. This approach enables IDE features to work on incomplete or incorrect code.

Use checkpoints and node wrapping to handle operator precedence and associativity. The checkpoint mechanism allows the parser to defer node creation until it has enough context to build the correct structure.

Preserve all source text including whitespace and comments. This lossless approach enables accurate source reconstruction and formatting preservation. Treat whitespace and comments as trivia tokens that the parser can skip when building the logical structure.

Cache tokenization results when implementing incremental parsing. Most edits affect only a small portion of the token stream, so reusing unchanged tokens significantly improves performance.

Build a typed AST layer over the raw syntax tree for ergonomic access. While the syntax tree provides complete information, the AST layer offers a more natural API for analysis and transformation tools.

The rowan architecture has proven highly successful in rust-analyzer, demonstrating that lossless syntax trees can provide both the precision needed for IDE features and the performance required for interactive use. The separation of green and red trees, combined with incremental reparsing, creates a solid foundation for modern language tooling.
