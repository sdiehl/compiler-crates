use std::fmt;

use chumsky::error::Rich;
use chumsky::extra;
use chumsky::prelude::*;

/// AST node for expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Identifier(String),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Call(String, Vec<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
}

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

/// Parse a simple expression language with operator precedence
pub fn expr_parser<'src>() -> impl Parser<'src, &'src str, Expr, extra::Err<Rich<'src, char>>> {
    let ident = text::ident().padded().to_slice();

    let number = text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .map(|s: &str| Expr::Number(s.parse().unwrap()))
        .padded();

    let atom = recursive(|expr| {
        let args = expr
            .clone()
            .separated_by(just(','))
            .allow_trailing()
            .collect()
            .delimited_by(just('('), just(')'));

        let call = ident
            .then(args)
            .map(|(name, args): (&str, Vec<Expr>)| Expr::Call(name.to_string(), args));

        let let_binding = text::keyword("let")
            .ignore_then(ident)
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(text::keyword("in"))
            .then(expr.clone())
            .map(|((name, value), body): ((&str, Expr), Expr)| {
                Expr::Let(name.to_string(), Box::new(value), Box::new(body))
            });

        choice((
            number,
            call,
            let_binding,
            ident.map(|s: &str| Expr::Identifier(s.to_string())),
            expr.delimited_by(just('('), just(')')),
        ))
    })
    .padded();

    let unary = just('-')
        .repeated()
        .collect::<Vec<_>>()
        .then(atom.clone())
        .map(|(ops, expr)| {
            ops.into_iter()
                .fold(expr, |expr, _| Expr::Unary(UnOp::Neg, Box::new(expr)))
        });

    let product = unary.clone().foldl(
        choice((just('*').to(BinOp::Mul), just('/').to(BinOp::Div)))
            .then(unary)
            .repeated(),
        |left, (op, right)| Expr::Binary(op, Box::new(left), Box::new(right)),
    );

    let sum = product.clone().foldl(
        choice((just('+').to(BinOp::Add), just('-').to(BinOp::Sub)))
            .then(product)
            .repeated(),
        |left, (op, right)| Expr::Binary(op, Box::new(left), Box::new(right)),
    );

    sum.then_ignore(end())
}

/// Token type for lexing
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Identifier(String),
    Keyword(String),
    Op(char),
    Delimiter(char),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Identifier(s) | Token::Keyword(s) => write!(f, "{}", s),
            Token::Op(c) | Token::Delimiter(c) => write!(f, "{}", c),
        }
    }
}

/// Lexer that produces tokens with spans
pub fn lexer<'src>(
) -> impl Parser<'src, &'src str, Vec<(Token, SimpleSpan)>, extra::Err<Rich<'src, char>>> {
    let number = text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .map(|s: &str| Token::Number(s.parse().unwrap()));

    let identifier = text::ident().to_slice().map(|s: &str| match s {
        "let" | "in" | "if" | "then" | "else" => Token::Keyword(s.to_string()),
        _ => Token::Identifier(s.to_string()),
    });

    let op = one_of("+-*/=<>!&|").map(Token::Op);
    let delimiter = one_of("(){}[],;").map(Token::Delimiter);

    let token = choice((number, identifier, op, delimiter)).padded_by(text::whitespace());

    token
        .map_with(|tok, e| (tok, e.span()))
        .repeated()
        .collect()
        .then_ignore(end())
}

/// Parser with error recovery
pub fn robust_parser<'src>() -> impl Parser<'src, &'src str, Vec<Expr>, extra::Err<Rich<'src, char>>>
{
    let ident = text::ident().padded().to_slice();

    let number = text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .map(|s: &str| Expr::Number(s.parse().unwrap_or(0.0)))
        .padded();

    let expr = recursive(|expr| {
        let atom = choice((
            number,
            ident.map(|s: &str| Expr::Identifier(s.to_string())),
            expr.clone()
                .delimited_by(just('('), just(')'))
                .recover_with(via_parser(nested_delimiters(
                    '(',
                    ')',
                    [('{', '}'), ('[', ']')],
                    |_| Expr::Number(0.0),
                ))),
        ));

        atom
    });

    expr.separated_by(just(';'))
        .allow_leading()
        .allow_trailing()
        .collect()
        .then_ignore(end())
}

/// Custom parser combinator for binary operators with precedence
pub fn binary_op_parser<'src>(
    ops: &[(&'src str, BinOp)],
    next: impl Parser<'src, &'src str, Expr, extra::Err<Rich<'src, char>>> + Clone + 'src,
) -> impl Parser<'src, &'src str, Expr, extra::Err<Rich<'src, char>>> + Clone + 'src {
    let op = choice(
        ops.iter()
            .map(|(s, op)| just(*s).to(op.clone()))
            .collect::<Vec<_>>(),
    );

    next.clone()
        .foldl(op.then(next).repeated(), |left, (op, right)| {
            Expr::Binary(op, Box::new(left), Box::new(right))
        })
}

/// Parser with custom error types
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken(String),
    UnclosedDelimiter(char),
    InvalidNumber(String),
}

pub fn validated_parser<'src>() -> impl Parser<'src, &'src str, Expr, extra::Err<Rich<'src, char>>>
{
    let number = text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .try_map(|s: &str, span| {
            s.parse::<f64>()
                .map(Expr::Number)
                .map_err(|_| Rich::custom(span, format!("Invalid number: {}", s)))
        });

    let ident = text::ident().to_slice().try_map(|s: &str, span| {
        if s.len() > 100 {
            Err(Rich::custom(span, "Identifier too long"))
        } else {
            Ok(Expr::Identifier(s.to_string()))
        }
    });

    choice((number, ident)).then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_parser() {
        let parser = expr_parser();

        let input = "2 + 3 * 4";
        let result = parser.parse(input);
        assert!(!result.has_errors());
        match result.into_output().unwrap() {
            Expr::Binary(BinOp::Add, left, right) => {
                assert_eq!(*left, Expr::Number(2.0));
                match *right {
                    Expr::Binary(BinOp::Mul, l, r) => {
                        assert_eq!(*l, Expr::Number(3.0));
                        assert_eq!(*r, Expr::Number(4.0));
                    }
                    _ => panic!("Expected multiplication on right"),
                }
            }
            _ => panic!("Expected addition at top level"),
        }
    }

    #[test]
    fn test_lexer() {
        let lexer = lexer();
        let input = "let x = 42 + 3.14";
        let result = lexer.parse(input);
        assert!(!result.has_errors());
        let tokens = result.into_output().unwrap();
        assert_eq!(tokens.len(), 6); // let, x, =, 42, +, 3.14
        assert_eq!(tokens[0].0, Token::Keyword("let".to_string()));
        assert_eq!(tokens[1].0, Token::Identifier("x".to_string()));
    }

    #[test]
    fn test_robust_parser() {
        let parser = robust_parser();

        // Test with valid input
        let input = "42; x; y";
        let result = parser.parse(input);
        assert!(!result.has_errors());
        assert_eq!(result.into_output().unwrap().len(), 3);

        // Test with recovery - unclosed paren
        let input_with_error = "42; (x; y";
        let result = parser.parse(input_with_error);
        // The parser should still produce some output even with errors
        assert!(result.has_errors());
    }

    #[test]
    fn test_binary_op_parser() {
        let ops = &[("&&", BinOp::Eq), ("||", BinOp::Eq)];

        let atom = text::int(10)
            .then(just('.').then(text::digits(10)).or_not())
            .to_slice()
            .map(|s: &str| Expr::Number(s.parse().unwrap()))
            .padded();

        let parser = binary_op_parser(ops, atom);
        let result = parser.parse("1 && 2 || 3");
        assert!(!result.has_errors());
    }

    #[test]
    fn test_validated_parser() {
        let parser = validated_parser();

        // Test valid input
        let result = parser.parse("42");
        assert!(!result.has_errors());
        assert_eq!(result.into_output().unwrap(), Expr::Number(42.0));

        // Test invalid number - this should produce an error
        let result = parser.parse("12.34.56");
        assert!(result.has_errors());
    }
}
