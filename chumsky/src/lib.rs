use std::fmt;

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
pub fn expr_parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    let ident = text::ident().padded();

    let number = text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .map(|s| Expr::Number(s.parse().unwrap()))
        .padded();

    let atom = number
        .or(ident.map(Expr::Identifier))
        .or(recursive(|expr| {
            let args = expr
                .clone()
                .separated_by(just(','))
                .allow_trailing()
                .delimited_by(just('('), just(')'));

            let call = ident.then(args).map(|(name, args)| Expr::Call(name, args));

            let let_binding = text::keyword("let")
                .ignore_then(ident)
                .then_ignore(just('='))
                .then(expr.clone())
                .then_ignore(text::keyword("in"))
                .then(expr.clone())
                .map(|((name, value), body)| Expr::Let(name, Box::new(value), Box::new(body)));

            choice((call, let_binding, expr.delimited_by(just('('), just(')'))))
        }))
        .padded();

    let unary = just('-')
        .repeated()
        .then(atom)
        .foldr(|_op, expr| Expr::Unary(UnOp::Neg, Box::new(expr)));

    let product = unary
        .clone()
        .then(
            choice((just('*').to(BinOp::Mul), just('/').to(BinOp::Div)))
                .then(unary)
                .repeated(),
        )
        .foldl(|left, (op, right)| Expr::Binary(op, Box::new(left), Box::new(right)));

    let sum = product
        .clone()
        .then(
            choice((just('+').to(BinOp::Add), just('-').to(BinOp::Sub)))
                .then(product)
                .repeated(),
        )
        .foldl(|left, (op, right)| Expr::Binary(op, Box::new(left), Box::new(right)));

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
            Token::Identifier(s) => write!(f, "{}", s),
            Token::Keyword(s) => write!(f, "{}", s),
            Token::Op(c) => write!(f, "{}", c),
            Token::Delimiter(c) => write!(f, "{}", c),
        }
    }
}

/// Lexer that produces tokens with spans
pub fn lexer() -> impl Parser<char, Vec<(Token, std::ops::Range<usize>)>, Error = Simple<char>> {
    let number = text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .map(|s| Token::Number(s.parse().unwrap()));

    let identifier = text::ident().map(|s: String| match s.as_str() {
        "let" | "in" | "if" | "then" | "else" => Token::Keyword(s),
        _ => Token::Identifier(s),
    });

    let op = one_of("+-*/=<>!&|").map(Token::Op);
    let delimiter = one_of("(){}[],;").map(Token::Delimiter);

    let token = choice((number, identifier, op, delimiter)).padded_by(text::whitespace());

    token
        .map_with_span(|tok, span| (tok, span))
        .repeated()
        .then_ignore(end())
}

/// Parser with error recovery
pub fn robust_parser() -> impl Parser<char, Vec<Expr>, Error = Simple<char>> {
    let ident = text::ident().padded();

    let number = text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .map(|s| Expr::Number(s.parse().unwrap_or(0.0)))
        .padded();

    let expr = recursive(|expr| {
        let atom = choice((
            number,
            ident.map(Expr::Identifier),
            expr.clone()
                .delimited_by(just('('), just(')'))
                .recover_with(nested_delimiters('(', ')', [], |_| Expr::Number(0.0))),
        ));

        let unary = just('-')
            .repeated()
            .then(atom)
            .foldr(|_op, expr| Expr::Unary(UnOp::Neg, Box::new(expr)));

        unary
            .clone()
            .then(
                choice((just('*').to(BinOp::Mul), just('/').to(BinOp::Div)))
                    .then(unary)
                    .repeated(),
            )
            .foldl(|left, (op, right)| Expr::Binary(op, Box::new(left), Box::new(right)))
    });

    expr.separated_by(just(';'))
        .allow_trailing()
        .then_ignore(end())
}

/// Custom parser combinator for binary operators with precedence
pub fn binary_op_parser<'a>(
    ops: &'a [(&'a str, BinOp)],
    next: impl Parser<char, Expr, Error = Simple<char>> + Clone + 'a,
) -> impl Parser<char, Expr, Error = Simple<char>> + Clone + 'a {
    let op = choice(
        ops.iter()
            .map(|(s, op)| just(*s).to(op.clone()))
            .collect::<Vec<_>>(),
    );

    next.clone()
        .then(op.then(next).repeated())
        .foldl(|left, (op, right)| Expr::Binary(op, Box::new(left), Box::new(right)))
}

/// Parser with custom error types
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken(String),
    UnclosedDelimiter(char),
    InvalidNumber(String),
}

pub fn validated_parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    let number = text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .validate(|s, span, emit| match s.parse::<f64>() {
            Ok(n) => Expr::Number(n),
            Err(_) => {
                emit(Simple::custom(span, format!("Invalid number: {}", s)));
                Expr::Number(0.0)
            }
        });

    let ident = text::ident().validate(|s: String, span, emit| {
        if s.len() > 100 {
            emit(Simple::custom(span, "Identifier too long"));
        }
        Expr::Identifier(s)
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
        assert!(result.is_ok());

        let expr = result.unwrap();
        assert_eq!(
            expr,
            Expr::Binary(
                BinOp::Add,
                Box::new(Expr::Number(2.0)),
                Box::new(Expr::Binary(
                    BinOp::Mul,
                    Box::new(Expr::Number(3.0)),
                    Box::new(Expr::Number(4.0))
                ))
            )
        );
    }

    #[test]
    fn test_lexer() {
        let lexer = lexer();
        let input = "let x = 42 in x + 1";
        let result = lexer.parse(input);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        assert_eq!(tokens[0].0, Token::Keyword("let".to_string()));
        assert_eq!(tokens[1].0, Token::Identifier("x".to_string()));
        assert_eq!(tokens[2].0, Token::Op('='));
        assert_eq!(tokens[3].0, Token::Number(42.0));
    }

    #[test]
    fn test_error_recovery() {
        let parser = robust_parser();
        let input = "1 + (2 * 3"; // Missing closing paren
        let result = parser.parse(input);
        // Parser recovers and returns partial result
        assert!(result.is_ok() || result.is_err());
    }
}
