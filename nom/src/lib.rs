use nom::branch::alt;
use nom::bytes::complete::{escaped, tag, take_while1};
use nom::character::complete::{alpha1, alphanumeric1, char, digit1, multispace0, one_of};
use nom::combinator::{map, map_res, opt, recognize, value};
use nom::multi::{fold_many0, many0, separated_list0};
use nom::sequence::{delimited, pair, preceded};
use nom::{IResult, Parser};

/// AST for a simple expression language
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Float(f64),
    String(String),
    Identifier(String),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
    Array(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Parse a floating-point number
pub fn float(input: &str) -> IResult<&str, f64> {
    map_res(
        recognize((opt(char('-')), digit1, opt((char('.'), digit1)))),
        |s: &str| s.parse::<f64>(),
    )
    .parse(input)
}

/// Parse an integer
pub fn integer(input: &str) -> IResult<&str, i64> {
    map_res(recognize(pair(opt(char('-')), digit1)), |s: &str| {
        s.parse::<i64>()
    })
    .parse(input)
}

/// Parse a string literal with escape sequences
pub fn string_literal(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        map(
            escaped(
                take_while1(|c: char| c != '"' && c != '\\'),
                '\\',
                one_of(r#""n\rt"#),
            ),
            |s: &str| s.to_string(),
        ),
        char('"'),
    )
    .parse(input)
}

/// Parse an identifier
pub fn identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| s.to_string(),
    )
    .parse(input)
}

/// Parse whitespace - wraps a parser with optional whitespace
fn ws<'a, O, F>(mut inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>, {
    move |input| {
        let (input, _) = multispace0.parse(input)?;
        let (input, result) = inner(input)?;
        let (input, _) = multispace0.parse(input)?;
        Ok((input, result))
    }
}

/// Parse a function call
pub fn function_call(input: &str) -> IResult<&str, Expr> {
    map(
        (
            |i| identifier.parse(i),
            ws(|i| {
                delimited(
                    char('('),
                    separated_list0(ws(|input| char(',').parse(input)), |i| expression.parse(i)),
                    char(')'),
                )
                .parse(i)
            }),
        ),
        |(name, args)| Expr::Call(name, args),
    )
    .parse(input)
}

/// Parse an array literal
pub fn array(input: &str) -> IResult<&str, Expr> {
    map(
        delimited(
            ws(|input| char('[').parse(input)),
            separated_list0(ws(|input| char(',').parse(input)), |i| expression.parse(i)),
            ws(|input| char(']').parse(input)),
        ),
        Expr::Array,
    )
    .parse(input)
}

/// Parse a primary expression
pub fn primary(input: &str) -> IResult<&str, Expr> {
    alt((
        map(|i| float.parse(i), Expr::Float),
        map(|i| integer.parse(i), Expr::Number),
        map(|i| string_literal.parse(i), Expr::String),
        |i| function_call.parse(i),
        |i| array.parse(i),
        map(|i| identifier.parse(i), Expr::Identifier),
        delimited(
            ws(|input| char('(').parse(input)),
            |i| expression.parse(i),
            ws(|input| char(')').parse(input)),
        ),
    ))
    .parse(input)
}

/// Parse a term (multiplication and division)
pub fn term(input: &str) -> IResult<&str, Expr> {
    let (input, init) = primary.parse(input)?;

    fold_many0(
        pair(
            ws(|input| {
                alt((value(BinOp::Mul, char('*')), value(BinOp::Div, char('/')))).parse(input)
            }),
            |i| primary.parse(i),
        ),
        move || init.clone(),
        |acc, (op, val)| Expr::Binary(op, Box::new(acc), Box::new(val)),
    )
    .parse(input)
}

/// Parse an expression (addition and subtraction)
pub fn expression(input: &str) -> IResult<&str, Expr> {
    let (input, init) = term.parse(input)?;

    fold_many0(
        pair(
            ws(|input| {
                alt((value(BinOp::Add, char('+')), value(BinOp::Sub, char('-')))).parse(input)
            }),
            |i| term.parse(i),
        ),
        move || init.clone(),
        |acc, (op, val)| Expr::Binary(op, Box::new(acc), Box::new(val)),
    )
    .parse(input)
}

/// Configuration file parser example
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub name: String,
    pub entries: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<Value>),
}

/// Parse a configuration value
pub fn config_value(input: &str) -> IResult<&str, Value> {
    alt((
        map(float, Value::Number),
        map(string_literal, Value::String),
        map(tag("true"), |_| Value::Boolean(true)),
        map(tag("false"), |_| Value::Boolean(false)),
        map(
            delimited(
                ws(|input| char('[').parse(input)),
                separated_list0(ws(|input| char(',').parse(input)), |i| {
                    config_value.parse(i)
                }),
                ws(|input| char(']').parse(input)),
            ),
            Value::List,
        ),
    ))
    .parse(input)
}

/// Parse a configuration entry
pub fn config_entry(input: &str) -> IResult<&str, (String, Value)> {
    map(
        (
            ws(|input| identifier.parse(input)),
            ws(|input| char('=').parse(input)),
            ws(|input| config_value.parse(input)),
        ),
        |(key, _, value)| (key, value),
    )
    .parse(input)
}

/// Parse a configuration section
pub fn config_section(input: &str) -> IResult<&str, Section> {
    map(
        (
            delimited(
                ws(|input| char('[').parse(input)),
                identifier,
                ws(|input| char(']').parse(input)),
            ),
            many0(config_entry),
        ),
        |(name, entries)| Section { name, entries },
    )
    .parse(input)
}

/// Parse a complete configuration file
pub fn parse_config(input: &str) -> IResult<&str, Config> {
    map(many0(ws(|input| config_section.parse(input))), |sections| {
        Config { sections }
    })
    .parse(input)
}

/// Custom error handling with context
pub fn parse_with_context(input: &str) -> IResult<&str, Expr> {
    alt((
        map(|i| float.parse(i), Expr::Float),
        map(|i| identifier.parse(i), Expr::Identifier),
        delimited(
            |i| delimited(multispace0, char('('), multispace0).parse(i),
            |i| parse_with_context.parse(i),
            |i| delimited(multispace0, char(')'), multispace0).parse(i),
        ),
    ))
    .parse(input)
}

/// Streaming parser for large files
pub fn streaming_parser(input: &str) -> IResult<&str, Vec<Expr>> {
    many0(delimited(
        |i| multispace0.parse(i),
        |i| expression.parse(i),
        |i| {
            alt((
                map(char(';'), |_| ()),
                map(|i2| multispace0.parse(i2), |_| ()),
            ))
            .parse(i)
        },
    ))
    .parse(input)
}

/// Binary format parser
pub fn parse_binary_header(input: &[u8]) -> IResult<&[u8], (u32, u32)> {
    use nom::number::complete::{be_u32, le_u32};

    (preceded(tag(&b"MAGIC"[..]), le_u32), be_u32).parse(input)
}

/// Parser with custom error type
#[derive(Debug, PartialEq)]
pub enum CustomError {
    InvalidNumber,
    UnexpectedToken,
    MissingDelimiter,
}

pub fn custom_error_parser(input: &str) -> IResult<&str, Expr> {
    alt((
        map(
            |i| float.parse(i),
            |n| {
                if n.is_finite() {
                    Expr::Float(n)
                } else {
                    Expr::Float(0.0) // Return default value for invalid numbers
                }
            },
        ),
        map(|i| identifier.parse(i), Expr::Identifier),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_parser() {
        assert_eq!(float.parse("3.15"), Ok(("", 3.15)));
        assert_eq!(float.parse("-2.5"), Ok(("", -2.5)));
        assert_eq!(float.parse("42"), Ok(("", 42.0)));
    }

    #[test]
    fn test_expression_parser() {
        use nom::Parser;
        let result = expression.parse("2 + 3 * 4").unwrap();
        assert_eq!(
            result.1,
            Expr::Binary(
                BinOp::Add,
                Box::new(Expr::Float(2.0)),
                Box::new(Expr::Binary(
                    BinOp::Mul,
                    Box::new(Expr::Float(3.0)),
                    Box::new(Expr::Float(4.0))
                ))
            )
        );
    }

    #[test]
    fn test_function_call() {
        use nom::Parser;
        let result = function_call.parse("max(1, 2, 3)").unwrap();
        assert_eq!(
            result.1,
            Expr::Call(
                "max".to_string(),
                vec![Expr::Float(1.0), Expr::Float(2.0), Expr::Float(3.0)]
            )
        );
    }

    #[test]
    fn test_config_parser() {
        use nom::Parser;
        let config = "[database]\nhost = \"localhost\"\nport = 5432\n";
        let result = parse_config.parse(config).unwrap();
        assert_eq!(result.1.sections.len(), 1);
        assert_eq!(result.1.sections[0].name, "database");
        assert_eq!(result.1.sections[0].entries.len(), 2);
    }
}
