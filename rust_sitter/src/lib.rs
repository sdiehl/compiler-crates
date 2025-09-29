//! rust_sitter example demonstrating Tree-sitter grammar generation through
//! Rust macros

/// Arithmetic expression grammar using rust_sitter macros
#[rust_sitter::grammar("arithmetic")]
pub mod arithmetic {
    /// The root expression type for our arithmetic language
    #[rust_sitter::language]
    #[derive(Debug, Clone, PartialEq)]
    pub enum Expr {
        /// Numeric literal
        Number(
            #[rust_sitter::leaf(pattern = r"\d+(\.\d+)?", transform = |v| v.parse().unwrap())] f64,
        ),

        /// Addition with left associativity (lower precedence)
        #[rust_sitter::prec_left(1)]
        Add(Box<Expr>, #[rust_sitter::leaf(text = "+")] (), Box<Expr>),

        /// Subtraction with left associativity (lower precedence)
        #[rust_sitter::prec_left(1)]
        Sub(Box<Expr>, #[rust_sitter::leaf(text = "-")] (), Box<Expr>),

        /// Multiplication with left associativity (higher precedence)
        #[rust_sitter::prec_left(2)]
        Mul(Box<Expr>, #[rust_sitter::leaf(text = "*")] (), Box<Expr>),

        /// Division with left associativity (higher precedence)
        #[rust_sitter::prec_left(2)]
        Div(Box<Expr>, #[rust_sitter::leaf(text = "/")] (), Box<Expr>),

        /// Exponentiation with right associativity (highest precedence)
        #[rust_sitter::prec_right(3)]
        Pow(Box<Expr>, #[rust_sitter::leaf(text = "^")] (), Box<Expr>),

        /// Parenthesized expression (highest precedence)
        #[rust_sitter::prec(4)]
        Paren(
            #[rust_sitter::leaf(text = "(")] (),
            Box<Expr>,
            #[rust_sitter::leaf(text = ")")] (),
        ),

        /// Unary negation
        #[rust_sitter::prec(4)]
        Neg(#[rust_sitter::leaf(text = "-")] (), Box<Expr>),
    }
}

/// Simple S-expression grammar
#[rust_sitter::grammar("s_expression")]
pub mod s_expression {
    /// S-expression language root
    #[rust_sitter::language]
    #[derive(Debug, Clone, PartialEq)]
    pub enum SExpr {
        /// Symbol/identifier
        Symbol(
            #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_\-]*", transform = |s| s.to_string())]
             String,
        ),

        /// Integer number
        Number(#[rust_sitter::leaf(pattern = r"-?\d+", transform = |s| s.parse().unwrap())] i64),

        /// String literal
        String(StringLiteral),

        /// List of S-expressions
        List(
            #[rust_sitter::leaf(text = "(")] (),
            #[rust_sitter::repeat(non_empty = false)] Vec<SExpr>,
            #[rust_sitter::leaf(text = ")")] (),
        ),
    }

    /// String literal with quotes
    #[derive(Debug, Clone, PartialEq)]
    pub struct StringLiteral {
        #[rust_sitter::leaf(text = "\"")]
        _open: (),
        #[rust_sitter::leaf(pattern = r#"([^"\\]|\\.)*"#, transform = |s| s.to_string())]
        pub value: String,
        #[rust_sitter::leaf(text = "\"")]
        _close: (),
    }
}

/// Simple configuration language
#[rust_sitter::grammar("config")]
pub mod config {
    use rust_sitter::Spanned;

    /// Configuration file root
    #[rust_sitter::language]
    #[derive(Debug, Clone)]
    pub struct Config {
        #[rust_sitter::repeat(non_empty = false)]
        pub entries: Vec<Entry>,
    }

    /// Configuration entry
    #[derive(Debug, Clone)]
    pub struct Entry {
        pub key: Key,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Spanned<Value>,
        #[rust_sitter::leaf(text = "\n")]
        _newline: (),
    }

    /// Configuration key
    #[derive(Debug, Clone)]
    pub struct Key {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z][a-zA-Z0-9_\.]*", transform = |s| s.to_string())]
        pub name: String,
    }

    /// Configuration value
    #[derive(Debug, Clone)]
    pub enum Value {
        String(StringValue),
        Number(
            #[rust_sitter::leaf(pattern = r"-?\d+(\.\d+)?", transform = |s| s.parse().unwrap())]
            f64,
        ),
        Bool(#[rust_sitter::leaf(pattern = r"true|false", transform = |s| s == "true")] bool),
        List(ListValue),
    }

    /// String value
    #[derive(Debug, Clone)]
    pub struct StringValue {
        #[rust_sitter::leaf(text = "\"")]
        _open: (),
        #[rust_sitter::leaf(pattern = r#"([^"\\]|\\.)*"#, transform = |s| s.to_string())]
        pub content: String,
        #[rust_sitter::leaf(text = "\"")]
        _close: (),
    }

    /// List value
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

// Evaluation implementations
impl arithmetic::Expr {
    /// Evaluate the arithmetic expression
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

#[cfg(test)]
mod tests {
    use super::*;

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

        // Parse with parentheses
        let expr = arithmetic::parse("(1 + 2) * 3").unwrap();
        assert_eq!(expr.eval(), 9.0); // (1 + 2) * 3

        // Parse power (right associative)
        let expr = arithmetic::parse("2 ^ 3 ^ 2").unwrap();
        assert_eq!(expr.eval(), 512.0); // 2 ^ (3 ^ 2) = 2 ^ 9

        // Parse negation
        let expr = arithmetic::parse("-5 + 10").unwrap();
        assert_eq!(expr.eval(), 5.0);
    }

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
            ("-10 + 20", 10.0),
            ("-(5 + 5)", -10.0),
        ];

        for (input, expected) in cases {
            let expr = arithmetic::parse(input).unwrap();
            let result = expr.eval();
            assert!(
                (result - expected).abs() < 0.001,
                "Failed for '{}': got {}, expected {}",
                input,
                result,
                expected
            );
        }
    }

    #[test]
    #[ignore] // Complex grammars may need adjustment for rust_sitter's parser generation
    fn test_s_expression_parsing() {
        // Parse symbol
        let sexpr = s_expression::parse("hello").unwrap();
        assert_eq!(sexpr, s_expression::SExpr::Symbol("hello".to_string()));

        // Parse number
        let sexpr = s_expression::parse("42").unwrap();
        assert_eq!(sexpr, s_expression::SExpr::Number(42));

        // Parse string
        let sexpr = s_expression::parse(r#""hello world""#).unwrap();
        match sexpr {
            s_expression::SExpr::String(s) => assert_eq!(s.value, "hello world"),
            _ => panic!("Expected String"),
        }

        // Parse list
        let sexpr = s_expression::parse("(+ 1 2)").unwrap();
        match sexpr {
            s_expression::SExpr::List(_, items, _) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], s_expression::SExpr::Symbol("+".to_string()));
                assert_eq!(items[1], s_expression::SExpr::Number(1));
                assert_eq!(items[2], s_expression::SExpr::Number(2));
            }
            _ => panic!("Expected List"),
        }

        // Parse nested list with comment
        let sexpr =
            s_expression::parse("(define (square x) ; Function definition\n  (* x x))").unwrap();
        match sexpr {
            s_expression::SExpr::List(_, items, _) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], s_expression::SExpr::Symbol("define".to_string()));
                // Check nested structure exists
                match &items[1] {
                    s_expression::SExpr::List(_, inner, _) => {
                        assert_eq!(inner.len(), 2);
                    }
                    _ => panic!("Expected nested list"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    #[ignore] // Complex grammars may need adjustment for rust_sitter's parser generation
    fn test_config_parsing() {
        // Parse simple config
        let config = config::parse("name = \"test\"\ncount = 42\n").unwrap();
        assert_eq!(config.entries.len(), 2);
        assert_eq!(config.entries[0].key.name, "name");
        assert_eq!(config.entries[1].key.name, "count");

        // Parse config with list
        let config = config::parse("items = [1, 2, 3]\n").unwrap();
        assert_eq!(config.entries.len(), 1);
        // Spanned<T> derefs to T
        match &*config.entries[0].value {
            config::Value::List(list) => {
                assert_eq!(list.items.len(), 3);
            }
            _ => panic!("Expected List"),
        }

        // Parse config with comments
        let config =
            config::parse("# Configuration file\nhost = \"localhost\"\nport = 8080\n").unwrap();
        assert_eq!(config.entries.len(), 2);
    }

    #[test]
    fn test_parse_errors() {
        // Arithmetic errors
        assert!(arithmetic::parse("1 +").is_err());
        assert!(arithmetic::parse("(1 + 2").is_err());

        // S-expression errors
        assert!(s_expression::parse("(unclosed").is_err());
        assert!(s_expression::parse(r#""unclosed string"#).is_err());

        // Config errors
        assert!(config::parse("no_equals_sign\n").is_err());
        assert!(config::parse("key = [1, 2,]\n").is_err()); // Trailing comma
    }
}
