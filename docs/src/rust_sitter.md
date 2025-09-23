# rust_sitter

rust_sitter provides a declarative approach to generating Tree-sitter parsers directly from Rust code. Unlike traditional parser generators that require separate grammar files, rust_sitter uses procedural macros to transform annotated Rust enums and structs into fully functional Tree-sitter parsers. This approach ensures type safety, enables IDE support, and keeps the grammar definition close to the AST types that consume it.

The library excels at creating incremental parsers suitable for editor integration, where parsing must handle incomplete or invalid input gracefully. The generated parsers support error recovery, incremental reparsing, and syntax highlighting through Tree-sitter's proven infrastructure. By defining grammars as Rust types, rust_sitter eliminates the impedance mismatch between grammar specifications and the data structures that represent parse trees.

## Basic Arithmetic Grammar

```rust
#[rust_sitter::grammar("arithmetic")]
pub mod arithmetic {
    #[rust_sitter::language]
    #[derive(Debug, Clone, PartialEq)]
    pub enum Expr {
        Number(
            #[rust_sitter::leaf(pattern = r"\d+(\.\d+)?", transform = |v| v.parse().unwrap())]
            f64,
        ),

        #[rust_sitter::prec_left(1)]
        Add(
            Box<Expr>,
            #[rust_sitter::leaf(text = "+")] (),
            Box<Expr>,
        ),

        #[rust_sitter::prec_left(2)]
        Mul(
            Box<Expr>,
            #[rust_sitter::leaf(text = "*")] (),
            Box<Expr>,
        ),

        #[rust_sitter::prec(4)]
        Paren(
            #[rust_sitter::leaf(text = "(")] (),
            Box<Expr>,
            #[rust_sitter::leaf(text = ")")] (),
        ),
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
```

The arithmetic module demonstrates fundamental grammar construction with operator precedence. The Expr enum represents different expression types, with each variant annotated to describe its parsing behavior. The leaf attribute defines terminal symbols, either matching exact text or patterns with optional transformations. Precedence annotations control parsing ambiguities, with higher numbers binding more tightly. Left-associative operators like addition and subtraction share the same precedence level, while right-associative exponentiation uses prec_right.

The Whitespace struct marked with the extra attribute defines tokens that can appear anywhere in the input without being part of the AST. This separation of structural and formatting concerns simplifies grammar definitions while maintaining flexibility in handling different coding styles.

## Expression Evaluation

```rust
impl arithmetic::Expr {
    pub fn eval(&self) -> f64 {
        match self {
            arithmetic::Expr::Number(n) => *n,
            arithmetic::Expr::Add(l, _, r) => l.eval() + r.eval(),
            arithmetic::Expr::Sub(l, _, r) => l.eval() - r.eval(),
            arithmetic::Expr::Mul(l, _, r) => l.eval() * r.eval(),
            arithmetic::Expr::Div(l, _, r) => l.eval() / r.eval(),
            arithmetic::Expr::Pow(l, _, r) => l.eval().powf(r.eval()),
            arithmetic::Expr::Paren(_, e, _) => e.eval(),
            arithmetic::Expr::Neg(_, e) => -e.eval(),
        }
    }
}
```

The eval method demonstrates how parsed ASTs integrate with Rust code. Since the grammar produces strongly-typed Rust values, implementing interpreters or compilers becomes straightforward pattern matching. The recursive structure naturally maps to recursive evaluation, with each expression type defining its semantic behavior.

## S-Expression Grammar

```rust
#[rust_sitter::grammar("s_expression")]
pub mod s_expression {
    #[rust_sitter::language]
    #[derive(Debug, Clone, PartialEq)]
    pub enum SExpr {
        Symbol(
            #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_\-]*", transform = |s| s.to_string())]
            String,
        ),

        Number(
            #[rust_sitter::leaf(pattern = r"-?\d+", transform = |s| s.parse().unwrap())]
            i64,
        ),

        String(StringLiteral),

        List(
            #[rust_sitter::leaf(text = "(")] (),
            #[rust_sitter::repeat(non_empty = false)]
            Vec<SExpr>,
            #[rust_sitter::leaf(text = ")")] (),
        ),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct StringLiteral {
        #[rust_sitter::leaf(text = "\"")]
        _open: (),
        #[rust_sitter::leaf(pattern = r#"([^"\\]|\\.)*"#, transform = |s| s.to_string())]
        pub value: String,
        #[rust_sitter::leaf(text = "\"")]
        _close: (),
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"[ \t\n\r]+")]
        _whitespace: (),
    }

    #[rust_sitter::extra]
    struct Comment {
        #[rust_sitter::leaf(pattern = r";[^\n]*")]
        _comment: (),
    }
}
```

The S-expression grammar showcases parsing of LISP-style symbolic expressions with nested lists and multiple data types. The SExpr enum includes symbols, numbers, strings, and lists. The repeat attribute with non_empty flag controls whether empty lists are allowed.

String parsing demonstrates pattern-based lexing with transformations. The pattern matches any sequence of non-quote characters or escape sequences, while the transform function converts the matched text into a Rust String. This separation of lexical and semantic concerns keeps grammars readable while supporting complex tokenization rules.

## Configuration Language Grammar

```rust
#[rust_sitter::grammar("config")]
pub mod config {
    use rust_sitter::Spanned;

    #[rust_sitter::language]
    #[derive(Debug, Clone)]
    pub struct Config {
        #[rust_sitter::repeat(non_empty = false)]
        pub entries: Vec<Entry>,
    }

    #[derive(Debug, Clone)]
    pub struct Entry {
        pub key: Key,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Spanned<Value>,
        #[rust_sitter::leaf(text = "\n")]
        _newline: (),
    }

    #[derive(Debug, Clone)]
    pub struct Key {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z][a-zA-Z0-9_\.]*", transform = |s| s.to_string())]
        pub name: String,
    }

    #[derive(Debug, Clone)]
    pub enum Value {
        String(StringValue),
        Number(
            #[rust_sitter::leaf(pattern = r"-?\d+(\.\d+)?", transform = |s| s.parse().unwrap())]
            f64,
        ),
        Bool(
            #[rust_sitter::leaf(pattern = r"true|false", transform = |s| s == "true")]
            bool,
        ),
        List(ListValue),
    }

    #[derive(Debug, Clone)]
    pub struct ListValue {
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        #[rust_sitter::repeat(non_empty = false)]
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub items: Vec<Value>,
        #[rust_sitter::leaf(text = "]")]
        _close: (),
    }
}
```

The config module implements a simple configuration file grammar with key-value pairs. The Config struct serves as the root node, containing a vector of entries. Each entry has a key, equals sign, value, and newline terminator. Values can be strings, numbers, booleans, or lists.

The Spanned type wraps values with source location information, useful for error reporting. Lists use the delimited attribute to handle comma-separated items. The extra whitespace and comment rules allow these tokens between any grammar elements.

This grammar demonstrates a practical use case for configuration files, showing how rust_sitter handles line-oriented formats with mixed value types.

## Grammar Annotations

rust_sitter provides several key annotations for controlling parser generation. The grammar attribute on a module specifies the parser name and generates the parse function. The language attribute marks the root type that parsing produces. Within types, leaf attributes define terminal symbols with optional patterns and transformations.

Precedence control uses prec, prec_left, and prec_right attributes with numeric levels. Higher numbers bind more tightly, resolving ambiguities in expression parsing. Associativity attributes determine how operators of the same precedence combine, critical for arithmetic and logical operations.

The repeat attribute generates zero-or-more or one-or-more patterns, with non_empty controlling minimums. Combined with delimited, it handles separated lists common in programming languages. The extra attribute marks ignorable tokens like whitespace and comments that can appear between any symbols.

## Error Recovery

Tree-sitter parsers excel at error recovery, continuing to parse even when encountering invalid syntax. This robustness makes them ideal for editor integration where code is frequently incomplete or temporarily malformed. The generated parser produces partial ASTs with error nodes marking problem areas, enabling features like syntax highlighting and code folding even in broken code.

Error nodes preserve the input text while marking parse failures, allowing tools to provide meaningful error messages. The incremental parsing capability means only changed regions require reparsing, maintaining responsiveness even in large files.

## Testing Patterns

```rust
#[test]
fn test_arithmetic_parsing() {
    // Parse simple number
    let expr = arithmetic::parse("42").unwrap();
    assert_eq!(expr, arithmetic::Expr::Number(42.0));

    // Parse addition
    let expr = arithmetic::parse("1 + 2").unwrap();
    match expr {
        arithmetic::Expr::Add(l, _, r) => {
            assert_eq!(*l, arithmetic::Expr::Number(1.0));
            assert_eq!(*r, arithmetic::Expr::Number(2.0));
        }
        _ => panic!("Expected Add"),
    }

    // Parse with precedence
    let expr = arithmetic::parse("1 + 2 * 3").unwrap();
    assert_eq!(expr.eval(), 7.0); // 1 + (2 * 3)
}
```

Parser testing combines unit tests for specific constructs with integration tests for complete programs. The parse function returns a Result, enabling standard Rust error handling. Comparing parsed ASTs with expected structures verifies parsing behavior, while evaluation tests confirm semantic correctness.

```rust
#[test]
fn test_arithmetic_evaluation() {
    let cases = vec![
        ("10.5 + 20.3", 30.8),
        ("100 - 50", 50.0),
        ("6 * 7", 42.0),
        ("20 / 4", 5.0),
        ("2 ^ 8", 256.0),
        ("(2 + 3) * (4 + 5)", 45.0),
        ("2 * 3 + 4 * 5", 26.0),
    ];

    for (input, expected) in cases {
        let expr = arithmetic::parse(input).unwrap();
        let result = expr.eval();
        assert!((result - expected).abs() < 0.001,
            "Failed for '{}': got {}, expected {}", input, result, expected);
    }
}
```

Property-based testing works well with grammar-based parsers. Generate random valid inputs according to the grammar, parse them, and verify properties like roundtrip printing or evaluation consistency. This approach finds edge cases that hand-written tests might miss.

## Build Configuration

The build.rs script integrates parser generation into the Cargo build process. The rust_sitter_tool::build_parsers function processes annotated modules, generating C code for Tree-sitter and Rust bindings. This generation happens at build time, ensuring parsers stay synchronized with their grammar definitions.

The generated code includes both the Tree-sitter parser tables and Rust wrapper functions. The parser tables use Tree-sitter's compact representation optimized for incremental parsing, while the wrapper provides safe Rust APIs matching the original type definitions.

## Integration with Tree-sitter

rust_sitter generates standard Tree-sitter parsers compatible with the entire Tree-sitter ecosystem. The parsers work with Tree-sitter's highlighting queries, language servers, and editor plugins. This compatibility means rust_sitter grammars can power syntax highlighting in editors like Neovim, VSCode, and Emacs without additional work.

The generated parsers support Tree-sitter's query language for pattern matching over syntax trees. Queries can extract specific patterns, power syntax highlighting, or identify code patterns for refactoring tools. This declarative approach to tree traversal complements the type-safe AST access from Rust code.

## Performance Characteristics

Tree-sitter parsers use table-driven parsing with excellent performance characteristics. The generated parsers handle gigabyte-scale files with sub-second parse times, while incremental reparsing typically completes in microseconds. Memory usage remains bounded even for large files through Tree-sitter's compressed tree representation.

The parsing algorithm uses a variant of LR parsing optimized for error recovery and incremental updates. Unlike traditional LR parsers that fail on first error, Tree-sitter continues parsing to produce useful partial results. This robustness comes with minimal performance overhead compared to strict parsers.

## Best Practices

Structure grammars to mirror the intended AST closely, using Rust's type system to enforce invariants. Separate lexical concerns using leaf patterns from structural concerns in the type hierarchy. Use precedence annotations consistently, documenting the intended associativity and binding strength.

Keep terminal patterns simple and unambiguous. Complex lexical rules should use separate leaf types rather than elaborate patterns. This separation improves error messages and makes grammars easier to understand and modify.

Design ASTs for consumption, not just parsing. Include semantic information in the types where it simplifies later processing. The transform functions on leaf nodes can perform initial interpretation, converting strings to numbers or normalizing identifiers.

Test grammars extensively with both valid and invalid inputs. Verify that error recovery produces useful partial results. Check that precedence and associativity match language specifications. Include tests for edge cases like empty inputs, deeply nested structures, and maximum-length identifiers.

rust_sitter bridges the gap between Tree-sitter's powerful parsing infrastructure and Rust's type system. By generating parsers from type definitions, it ensures grammar and AST remain synchronized while providing excellent IDE support and type safety. The combination of declarative grammar specification, automatic parser generation, and Tree-sitter's robust parsing algorithm makes rust_sitter an excellent choice for building development tools and language implementations.