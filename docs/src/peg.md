# peg

The `peg` crate provides a parser generator based on Parsing Expression Grammars (PEGs). PEGs offer a powerful alternative to traditional parsing approaches, combining the ease of writing recursive descent parsers with the declarative nature of grammar specifications. Unlike context-free grammars used by tools like yacc or LALR parsers, PEGs are unambiguous by design - the first matching alternative always wins, eliminating shift/reduce conflicts.

For compiler construction, peg excels at rapidly prototyping language parsers. The grammar syntax closely mirrors how you think about your language's structure, and the generated parser includes automatic error reporting with position information. PEGs handle unlimited lookahead naturally and support semantic actions directly in the grammar, making it straightforward to build ASTs during parsing.

## Grammar Definition

PEG grammars are defined using Rust macros that generate efficient parsers at compile time:

```rust
#![module!("peg/src/lib.rs", language)]
```

This concise grammar definition expands into a complete recursive descent parser with error handling and position tracking.

## Expression Parsing

The grammar handles expressions with proper operator precedence:

```rust
#![enum!("peg/src/lib.rs", Expr)]
```

```rust
#![enum!("peg/src/lib.rs", BinaryOp)]
```

The precedence climbing in the grammar ensures that `1 + 2 * 3` parses as `1 + (2 * 3)` rather than `(1 + 2) * 3`.

## Literal Parsing

PEG excels at parsing various literal formats with precise control:

```rust
#![function!("peg/src/lib.rs", parse_expression)]
```

The grammar handles integers, floats, strings with escape sequences, booleans, and identifiers with appropriate validation rules.

## Function Calls and Lambda Expressions

Parsing function application and lambda expressions demonstrates PEG's ability to handle complex nested structures:

The grammar supports both simple function calls like `f(x, y)` and curried application like `f x y`, as well as lambda expressions with multiple parameters.

## Let Expressions and Conditionals

Structured expressions show how PEG handles keyword-based constructs:

The `let` and `if` expressions demonstrate how PEG naturally handles indentation-insensitive syntax with clear keyword boundaries.

## Lists and Records

Collection types showcase PEG's repetition and separator handling:

The grammar uses PEG's repetition operators (`**` for separated lists) to elegantly handle comma-separated values with optional trailing commas.

## Program Structure

A complete program consists of multiple statements:

```rust
#![enum!("peg/src/lib.rs", Statement)]
```

```rust
#![struct!("peg/src/lib.rs", Program)]
```

```rust
#![function!("peg/src/lib.rs", parse_program)]
```

This structure supports mixing expressions, definitions, and type declarations in a program.

## Type Definitions

The grammar includes support for algebraic data types through the Statement enum:

```rust
#![enum!("peg/src/lib.rs", Statement)]
```

Type definitions enable pattern matching and custom data structures in the parsed language. The `TypeDef` variant stores the type name and its constructors with their parameter types.

## Comment Handling

PEG makes it easy to handle both line and block comments:

Comments are automatically skipped in whitespace, simplifying the rest of the grammar.

## Expression Evaluation

A simple evaluator demonstrates working with the parsed AST:

```rust
#![function!("peg/src/lib.rs", evaluate)]
```

This evaluator handles basic arithmetic operations with proper error handling for cases like division by zero.

## Error Reporting

PEG automatically generates error messages with position information:

```rust
#![test!("peg/src/lib.rs", test_error_handling)]
```

The generated parser tracks the furthest position reached and expected tokens, providing helpful error messages for syntax errors.

## Precedence and Associativity

The grammar correctly handles operator precedence through its structure:

```rust
#![test!("peg/src/lib.rs", test_precedence)]
```

Higher-precedence operators are parsed in deeper rules, ensuring correct parse trees without ambiguity.

## Advanced Grammar Features

PEG supports several advanced features useful for compiler construction:

**Syntactic Predicates**: Use `&` for positive lookahead and `!` for negative lookahead without consuming input.

**Semantic Actions**: Embed Rust code directly in the grammar to build ASTs or perform validation during parsing.

**Rule Parameters**: Pass parameters to rules for context-sensitive parsing.

**Position Tracking**: Access the current position in the input for error reporting or source mapping.

**Custom Error Types**: Define your own error types for domain-specific error reporting.

## Performance Characteristics

PEG parsers have predictable performance characteristics:

**Linear Time**: PEGs parse in linear time with memoization (packrat parsing) or near-linear without.

**Memory Usage**: Packrat parsing trades memory for guaranteed linear time by memoizing all rule applications.

**No Backtracking**: Despite appearances, well-written PEG grammars minimize backtracking through careful ordering of alternatives.

**Direct Execution**: The generated parser is direct Rust code, avoiding interpretation overhead.

## Grammar Design Best Practices

Structure your PEG grammar for clarity and performance:

Order alternatives from most specific to least specific. Since PEGs use ordered choice, put more specific patterns first to avoid incorrect matches.

Factor out common prefixes to reduce redundant parsing. Instead of `"if" / "ifx"`, use `"if" "x"?`.

Use cut operators (`@`) to commit to a parse once certain syntax is recognized, improving error messages.

Keep semantic actions simple. Complex AST construction is better done in a separate pass.

Design for positive matching rather than negative. PEGs work best when describing what syntax looks like, not what it doesn't.
