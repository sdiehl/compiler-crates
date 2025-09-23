use std::ops::Range;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::{char, digit1, multispace0, multispace1};
use nom::combinator::{map, recognize};
use nom::error::{Error, ErrorKind};
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, preceded};
use nom::{Err, IResult, Parser as NomParser};
use nom_locate::{position, LocatedSpan};

/// A span type that tracks position information
pub type Span<'a> = LocatedSpan<&'a str>;

/// Location information extracted from a span
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub line: u32,
    pub column: usize,
    pub offset: usize,
}

impl Location {
    pub fn from_span(span: Span<'_>) -> Self {
        Self {
            line: span.location_line(),
            column: span.get_utf8_column(),
            offset: span.location_offset(),
        }
    }
}

/// A range of source locations
#[derive(Debug, Clone, PartialEq)]
pub struct SourceRange {
    pub start: Location,
    pub end: Location,
}

impl SourceRange {
    pub fn from_spans(start: Span<'_>, end: Span<'_>) -> Self {
        Self {
            start: Location::from_span(start),
            end: Location::from_span(end),
        }
    }

    pub fn to_range(&self) -> Range<usize> {
        self.start.offset..self.end.offset
    }
}

/// AST node with location information
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: SourceRange,
}

impl<T> Spanned<T> {
    pub fn new(node: T, start: Span<'_>, end: Span<'_>) -> Self {
        Self {
            node,
            span: SourceRange::from_spans(start, end),
        }
    }
}

/// Expression AST for a simple language
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Identifier(String),
    Binary {
        left: Box<Spanned<Expr>>,
        op: BinaryOp,
        right: Box<Spanned<Expr>>,
    },
    Call {
        func: Box<Spanned<Expr>>,
        args: Vec<Spanned<Expr>>,
    },
    Let {
        name: String,
        value: Box<Spanned<Expr>>,
        body: Box<Spanned<Expr>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
}

/// Parser error with precise location information
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub location: Location,
    pub expected: Vec<String>,
}

impl ParseError {
    pub fn from_nom_error(_input: Span<'_>, error: Error<Span<'_>>) -> Self {
        let location = Location::from_span(error.input);
        let message = match error.code {
            ErrorKind::Tag => "unexpected token".to_string(),
            ErrorKind::Digit => "expected number".to_string(),
            ErrorKind::Alpha => "expected identifier".to_string(),
            ErrorKind::Char => "expected character".to_string(),
            ErrorKind::Many0 => "expected list".to_string(),
            ErrorKind::SeparatedList => "expected comma-separated list".to_string(),
            _ => format!("parse error: {:?}", error.code),
        };

        Self {
            message,
            location,
            expected: vec![],
        }
    }
}

/// Main parser implementation
pub struct Parser;

impl Parser {
    /// Parse a complete expression from input
    pub fn parse_expression(input: &str) -> Result<Spanned<Expr>, ParseError> {
        let span = Span::new(input);
        match Self::expression(span) {
            Ok((_, expr)) => Ok(expr),
            Err(Err::Error(e)) | Err(Err::Failure(e)) => Err(ParseError::from_nom_error(span, e)),
            Err(Err::Incomplete(_)) => Err(ParseError {
                message: "incomplete input".to_string(),
                location: Location::from_span(span),
                expected: vec!["more input".to_string()],
            }),
        }
    }

    /// Parse an expression with precedence
    fn expression(input: Span<'_>) -> IResult<Span<'_>, Spanned<Expr>> {
        Self::binary_expr(input, 0)
    }

    /// Parse binary expressions with operator precedence
    fn binary_expr(input: Span<'_>, min_prec: u8) -> IResult<Span<'_>, Spanned<Expr>> {
        let start_pos = position(input)?;
        let (input, mut left) = Self::primary_expr(input)?;

        let mut current_input = input;
        loop {
            let (input, _) = multispace0(current_input)?;

            // Try to parse an operator
            let op_result: IResult<Span<'_>, (BinaryOp, u8)> = alt((
                map(char('+'), |_| (BinaryOp::Add, 1)),
                map(char('-'), |_| (BinaryOp::Sub, 1)),
                map(char('*'), |_| (BinaryOp::Mul, 2)),
                map(char('/'), |_| (BinaryOp::Div, 2)),
                map(tag("=="), |_| (BinaryOp::Eq, 0)),
                map(char('<'), |_| (BinaryOp::Lt, 0)),
                map(char('>'), |_| (BinaryOp::Gt, 0)),
            ))
            .parse(input);

            match op_result {
                Ok((input, (op, prec))) if prec >= min_prec => {
                    let (input, _) = multispace0(input)?;
                    let (input, right) = Self::binary_expr(input, prec + 1)?;

                    let end_span = position(input)?;
                    left = Spanned::new(
                        Expr::Binary {
                            left: Box::new(left),
                            op,
                            right: Box::new(right),
                        },
                        start_pos.0,
                        end_span.0,
                    );
                    current_input = input;
                }
                _ => break,
            }
        }

        Ok((current_input, left))
    }

    /// Parse primary expressions (atoms and parenthesized expressions)
    fn primary_expr(input: Span<'_>) -> IResult<Span<'_>, Spanned<Expr>> {
        let (input, _) = multispace0(input)?;

        alt((
            Self::parenthesized_expr,
            Self::function_call,
            Self::let_expr,
            Self::number,
            Self::identifier,
        ))
        .parse(input)
    }

    /// Parse parenthesized expressions
    fn parenthesized_expr(input: Span<'_>) -> IResult<Span<'_>, Spanned<Expr>> {
        let start_pos = position(input)?;
        let (input, expr) = delimited(
            char('('),
            preceded(multispace0, Self::expression),
            preceded(multispace0, char(')')),
        )
        .parse(input)?;
        let end_pos = position(input)?;

        Ok((input, Spanned::new(expr.node, start_pos.0, end_pos.0)))
    }

    /// Parse function calls
    fn function_call(input: Span<'_>) -> IResult<Span<'_>, Spanned<Expr>> {
        let start_pos = position(input)?;
        let (input, func) = Self::identifier(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char('(')(input)?;
        let (input, _) = multispace0(input)?;
        let (input, args) = separated_list0(
            delimited(multispace0, char(','), multispace0),
            Self::expression,
        )
        .parse(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char(')')(input)?;
        let end_pos = position(input)?;

        Ok((
            input,
            Spanned::new(
                Expr::Call {
                    func: Box::new(func),
                    args,
                },
                start_pos.0,
                end_pos.0,
            ),
        ))
    }

    /// Parse let expressions
    fn let_expr(input: Span<'_>) -> IResult<Span<'_>, Spanned<Expr>> {
        let start_pos = position(input)?;
        let (input, _) = tag("let")(input)?;
        let (input, _) = multispace1(input)?;
        let (input, name) = Self::identifier_string(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char('=')(input)?;
        let (input, _) = multispace0(input)?;
        let (input, value) = Self::expression(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = tag("in")(input)?;
        let (input, _) = multispace0(input)?;
        let (input, body) = Self::expression(input)?;
        let end_pos = position(input)?;

        Ok((
            input,
            Spanned::new(
                Expr::Let {
                    name,
                    value: Box::new(value),
                    body: Box::new(body),
                },
                start_pos.0,
                end_pos.0,
            ),
        ))
    }

    /// Parse numbers
    fn number(input: Span<'_>) -> IResult<Span<'_>, Spanned<Expr>> {
        let start_pos = position(input)?;
        let (input, digits) = digit1(input)?;
        let end_pos = position(input)?;

        let num = digits
            .fragment()
            .parse()
            .map_err(|_| Err::Error(Error::new(input, ErrorKind::Digit)))?;

        Ok((
            input,
            Spanned::new(Expr::Number(num), start_pos.0, end_pos.0),
        ))
    }

    /// Parse identifiers
    fn identifier(input: Span<'_>) -> IResult<Span<'_>, Spanned<Expr>> {
        let start_pos = position(input)?;
        let (input, ident) = Self::identifier_string(input)?;
        let end_pos = position(input)?;

        Ok((
            input,
            Spanned::new(Expr::Identifier(ident), start_pos.0, end_pos.0),
        ))
    }

    /// Parse identifier strings
    fn identifier_string(input: Span<'_>) -> IResult<Span<'_>, String> {
        let (input, ident) = recognize(pair(
            alt((tag("_"), take_while1(|c: char| c.is_ascii_alphabetic()))),
            take_while(|c: char| c.is_ascii_alphanumeric() || c == '_'),
        ))
        .parse(input)?;

        Ok((input, ident.fragment().to_string()))
    }

    /// Get position information for error reporting
    pub fn get_position_info(input: &str, offset: usize) -> Option<(u32, usize)> {
        if offset <= input.len() {
            // Count lines and column up to the offset
            let mut line = 1;
            let mut col = 1;

            for (i, ch) in input.char_indices() {
                if i >= offset {
                    break;
                }
                if ch == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
            }

            Some((line, col))
        } else {
            None
        }
    }

    /// Extract line content for error reporting
    pub fn get_line_content(input: &str, line_number: u32) -> Option<&str> {
        input.lines().nth(line_number.saturating_sub(1) as usize)
    }
}

/// A lexer that preserves location information for each token
pub struct LocatedLexer<'a> {
    input: Span<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocatedToken {
    pub kind: TokenKind,
    pub location: Location,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number,
    Identifier,
    Keyword(String),
    Operator(String),
    LeftParen,
    RightParen,
    Comma,
    Equals,
    Whitespace,
    Comment,
    Eof,
}

impl<'a> LocatedLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: Span::new(input),
        }
    }

    /// Tokenize input while preserving location information
    pub fn tokenize(&mut self) -> Result<Vec<LocatedToken>, ParseError> {
        let mut tokens = Vec::new();
        let mut current = self.input;

        while !current.fragment().is_empty() {
            let _start_pos = position(current).map_err(|e| {
                ParseError::from_nom_error(
                    current,
                    match e {
                        Err::Error(err) | Err::Failure(err) => err,
                        Err::Incomplete(_) => Error::new(current, ErrorKind::Complete),
                    },
                )
            })?;

            let (remaining, token) = self.next_token(current).map_err(|e| match e {
                Err::Error(err) | Err::Failure(err) => ParseError::from_nom_error(current, err),
                Err::Incomplete(_) => ParseError {
                    message: "incomplete token".to_string(),
                    location: Location::from_span(current),
                    expected: vec!["complete token".to_string()],
                },
            })?;

            if let Some(token) = token {
                tokens.push(token);
            }

            current = remaining;
        }

        tokens.push(LocatedToken {
            kind: TokenKind::Eof,
            location: Location::from_span(current),
            text: String::new(),
        });

        Ok(tokens)
    }

    fn next_token(&self, input: Span<'a>) -> IResult<Span<'a>, Option<LocatedToken>> {
        alt((
            map(|i| self.whitespace_or_comment(i), |_| None),
            map(|i| self.keyword_or_identifier(i), Some),
            map(|i| self.number_token(i), Some),
            map(|i| self.operator_token(i), Some),
            map(|i| self.punctuation_token(i), Some),
        ))
        .parse(input)
    }

    fn whitespace_or_comment(&self, input: Span<'a>) -> IResult<Span<'a>, ()> {
        let (input, _) = alt((
            multispace1,
            recognize((tag("//"), take_while(|c| c != '\n'))),
        ))
        .parse(input)?;
        Ok((input, ()))
    }

    fn keyword_or_identifier(&self, input: Span<'a>) -> IResult<Span<'a>, LocatedToken> {
        let start_pos = position(input)?;
        let (input, ident) = recognize(pair(
            alt((tag("_"), take_while1(|c: char| c.is_ascii_alphabetic()))),
            take_while(|c: char| c.is_ascii_alphanumeric() || c == '_'),
        ))
        .parse(input)?;

        let text = ident.fragment().to_string();
        let kind = match text.as_str() {
            "let" | "in" | "if" | "then" | "else" | "fn" => TokenKind::Keyword(text.clone()),
            _ => TokenKind::Identifier,
        };

        Ok((
            input,
            LocatedToken {
                kind,
                location: Location::from_span(start_pos.0),
                text,
            },
        ))
    }

    fn number_token(&self, input: Span<'a>) -> IResult<Span<'a>, LocatedToken> {
        let start_pos = position(input)?;
        let (input, number) = digit1(input)?;

        Ok((
            input,
            LocatedToken {
                kind: TokenKind::Number,
                location: Location::from_span(start_pos.0),
                text: number.fragment().to_string(),
            },
        ))
    }

    fn operator_token(&self, input: Span<'a>) -> IResult<Span<'a>, LocatedToken> {
        let start_pos = position(input)?;
        let (input, op) = alt((
            tag("=="),
            tag("<="),
            tag(">="),
            tag("!="),
            tag("+"),
            tag("-"),
            tag("*"),
            tag("/"),
            tag("<"),
            tag(">"),
        ))
        .parse(input)?;

        Ok((
            input,
            LocatedToken {
                kind: TokenKind::Operator(op.fragment().to_string()),
                location: Location::from_span(start_pos.0),
                text: op.fragment().to_string(),
            },
        ))
    }

    fn punctuation_token(&self, input: Span<'a>) -> IResult<Span<'a>, LocatedToken> {
        let start_pos = position(input)?;
        let (input, punct) = alt((
            map(char('('), |_| TokenKind::LeftParen),
            map(char(')'), |_| TokenKind::RightParen),
            map(char(','), |_| TokenKind::Comma),
            map(char('='), |_| TokenKind::Equals),
        ))
        .parse(input)?;

        let text = match punct {
            TokenKind::LeftParen => "(".to_string(),
            TokenKind::RightParen => ")".to_string(),
            TokenKind::Comma => ",".to_string(),
            TokenKind::Equals => "=".to_string(),
            _ => String::new(),
        };

        Ok((
            input,
            LocatedToken {
                kind: punct,
                location: Location::from_span(start_pos.0),
                text,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_tracking() {
        let input = "let x = 42\nin y + 1";
        let span = Span::new(input);

        // Test line and column calculation
        assert_eq!(span.location_line(), 1);
        assert_eq!(span.get_utf8_column(), 1);

        // Test position after advancing
        let (remaining, _): (Span, Span) = tag::<&str, Span, Error<Span>>("let")(span).unwrap();
        assert_eq!(remaining.location_offset(), 3);
    }

    #[test]
    fn test_number_parsing() {
        let result = Parser::parse_expression("42").unwrap();
        assert_eq!(result.node, Expr::Number(42));
        assert_eq!(result.span.start.line, 1);
        assert_eq!(result.span.start.column, 1);
    }

    #[test]
    fn test_binary_expression() {
        let result = Parser::parse_expression("2 + 3 * 4").unwrap();

        if let Expr::Binary { left, op, right } = result.node {
            assert_eq!(op, BinaryOp::Add);
            assert_eq!(left.node, Expr::Number(2));

            if let Expr::Binary { left, op, right } = right.node {
                assert_eq!(op, BinaryOp::Mul);
                assert_eq!(left.node, Expr::Number(3));
                assert_eq!(right.node, Expr::Number(4));
            } else {
                panic!("Expected binary expression for multiplication");
            }
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_function_call_parsing() {
        let result = Parser::parse_expression("add(1, 2)").unwrap();

        if let Expr::Call { func, args } = result.node {
            assert_eq!(func.node, Expr::Identifier("add".to_string()));
            assert_eq!(args.len(), 2);
            assert_eq!(args[0].node, Expr::Number(1));
            assert_eq!(args[1].node, Expr::Number(2));
        } else {
            panic!("Expected function call");
        }
    }

    #[test]
    fn test_let_expression() {
        let result = Parser::parse_expression("let x = 5 in x + 1").unwrap();

        if let Expr::Let { name, value, body } = result.node {
            assert_eq!(name, "x");
            assert_eq!(value.node, Expr::Number(5));

            if let Expr::Binary { left, op, right } = body.node {
                assert_eq!(op, BinaryOp::Add);
                assert_eq!(left.node, Expr::Identifier("x".to_string()));
                assert_eq!(right.node, Expr::Number(1));
            } else {
                panic!("Expected binary expression in let body");
            }
        } else {
            panic!("Expected let expression");
        }
    }

    #[test]
    fn test_error_location() {
        let result = Parser::parse_expression("2 + ");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.location.line, 1);
        assert_eq!(error.location.column, 5);
    }

    #[test]
    fn test_lexer_with_locations() {
        let mut lexer = LocatedLexer::new("let x = 42");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 5); // let, x, =, 42, EOF
        assert_eq!(tokens[0].kind, TokenKind::Keyword("let".to_string()));
        assert_eq!(tokens[0].location.column, 1);

        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].text, "x");
        assert_eq!(tokens[1].location.column, 5);

        assert_eq!(tokens[2].kind, TokenKind::Equals);
        assert_eq!(tokens[2].location.column, 7);
    }

    #[test]
    fn test_multiline_locations() {
        let input = "let x = 1\nlet y = 2";
        let mut lexer = LocatedLexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        // Find the second 'let' token
        let second_let = tokens
            .iter()
            .find(|t| {
                matches!(t.kind, TokenKind::Keyword(ref k) if k == "let") && t.location.line == 2
            })
            .expect("Should find second let token");

        assert_eq!(second_let.location.line, 2);
        assert_eq!(second_let.location.column, 1);
    }

    #[test]
    fn test_position_helpers() {
        let input = "line1\nline2\nline3";

        let pos_info = Parser::get_position_info(input, 7); // Position of 'l' in "line2"
        assert_eq!(pos_info, Some((2, 2)));

        let line_content = Parser::get_line_content(input, 2);
        assert_eq!(line_content, Some("line2"));
    }

    #[test]
    fn test_source_range() {
        let result = Parser::parse_expression("42 + 3").unwrap();
        let range = result.span.to_range();
        assert_eq!(range, 0..6);
    }
}
