use winnow::ascii::{alpha1, digit1, multispace0, space0};
use winnow::combinator::{alt, delimited, preceded, repeat, separated, terminated};
use winnow::token::{take_till, take_while};
use winnow::Parser;

type PResult<T> = Result<T, winnow::error::ErrMode<winnow::error::ContextError>>;

// Arithmetic Expression Parser

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Paren(Box<Expr>),
}

impl Expr {
    pub fn eval(&self) -> f64 {
        match self {
            Expr::Number(n) => *n,
            Expr::Add(a, b) => a.eval() + b.eval(),
            Expr::Sub(a, b) => a.eval() - b.eval(),
            Expr::Mul(a, b) => a.eval() * b.eval(),
            Expr::Div(a, b) => a.eval() / b.eval(),
            Expr::Paren(e) => e.eval(),
        }
    }
}

pub fn parse_expression(input: &str) -> Result<Expr, String> {
    expr.parse(input).map_err(|e| e.to_string())
}

fn expr(input: &mut &str) -> PResult<Expr> {
    add_sub(input)
}

fn add_sub(input: &mut &str) -> PResult<Expr> {
    let init = mul_div(input)?;

    repeat(0.., (delimited(space0, alt(('+', '-')), space0), mul_div))
        .fold(
            move || init.clone(),
            |acc, (op, val)| match op {
                '+' => Expr::Add(Box::new(acc), Box::new(val)),
                '-' => Expr::Sub(Box::new(acc), Box::new(val)),
                _ => unreachable!(),
            },
        )
        .parse_next(input)
}

fn mul_div(input: &mut &str) -> PResult<Expr> {
    let init = factor(input)?;

    repeat(0.., (delimited(space0, alt(('*', '/')), space0), factor))
        .fold(
            move || init.clone(),
            |acc, (op, val)| match op {
                '*' => Expr::Mul(Box::new(acc), Box::new(val)),
                '/' => Expr::Div(Box::new(acc), Box::new(val)),
                _ => unreachable!(),
            },
        )
        .parse_next(input)
}

fn factor(input: &mut &str) -> PResult<Expr> {
    alt((
        number.map(Expr::Number),
        delimited('(', preceded(space0, expr), preceded(space0, ')'))
            .map(|e| Expr::Paren(Box::new(e))),
    ))
    .parse_next(input)
}

fn number(input: &mut &str) -> PResult<f64> {
    take_while(1.., |c: char| c.is_ascii_digit() || c == '.')
        .try_map(|s: &str| s.parse::<f64>())
        .parse_next(input)
}

// JSON Parser

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

pub fn parse_json(input: &str) -> Result<Json, String> {
    delimited(multispace0, json_value, multispace0)
        .parse(input.trim())
        .map_err(|e| e.to_string())
}

fn json_value(input: &mut &str) -> PResult<Json> {
    delimited(
        multispace0,
        alt((
            "null".value(Json::Null),
            "true".value(Json::Bool(true)),
            "false".value(Json::Bool(false)),
            json_number,
            json_string.map(Json::String),
            json_array,
            json_object,
        )),
        multispace0,
    )
    .parse_next(input)
}

fn json_number(input: &mut &str) -> PResult<Json> {
    take_while(1.., |c: char| {
        c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E' || c == '+'
    })
    .try_map(|s: &str| s.parse::<f64>().map(Json::Number))
    .parse_next(input)
}

fn json_string(input: &mut &str) -> PResult<String> {
    delimited('"', take_till(0.., '"').map(|s: &str| s.to_string()), '"').parse_next(input)
}

fn json_array(input: &mut &str) -> PResult<Json> {
    delimited(
        '[',
        delimited(
            multispace0,
            separated(0.., json_value, delimited(multispace0, ',', multispace0)),
            multispace0,
        ),
        ']',
    )
    .map(Json::Array)
    .parse_next(input)
}

fn json_object(input: &mut &str) -> PResult<Json> {
    delimited(
        '{',
        delimited(
            multispace0,
            separated(0.., json_member, delimited(multispace0, ',', multispace0)),
            multispace0,
        ),
        '}',
    )
    .map(Json::Object)
    .parse_next(input)
}

fn json_member(input: &mut &str) -> PResult<(String, Json)> {
    (
        terminated(json_string, delimited(multispace0, ':', multispace0)),
        json_value,
    )
        .parse_next(input)
}

// S-Expression Parser

#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Symbol(String),
    Number(i64),
    String(String),
    List(Vec<SExpr>),
}

pub fn parse_sexpr(input: &str) -> Result<SExpr, String> {
    sexpr_value.parse(input).map_err(|e| e.to_string())
}

fn sexpr_value(input: &mut &str) -> PResult<SExpr> {
    delimited(
        sexpr_ws,
        alt((sexpr_number, sexpr_string, sexpr_symbol, sexpr_list)),
        sexpr_ws,
    )
    .parse_next(input)
}

fn sexpr_ws(input: &mut &str) -> PResult<()> {
    take_while(0.., |c: char| c.is_ascii_whitespace())
        .void()
        .parse_next(input)
}

fn sexpr_symbol(input: &mut &str) -> PResult<SExpr> {
    take_while(1.., |c: char| {
        c.is_ascii_alphanumeric()
            || c == '_'
            || c == '-'
            || c == '+'
            || c == '*'
            || c == '/'
            || c == '?'
    })
    .map(|s: &str| SExpr::Symbol(s.to_string()))
    .parse_next(input)
}

fn sexpr_number(input: &mut &str) -> PResult<SExpr> {
    (winnow::combinator::opt('-'), digit1)
        .take()
        .try_map(|s: &str| s.parse::<i64>().map(SExpr::Number))
        .parse_next(input)
}

fn sexpr_string(input: &mut &str) -> PResult<SExpr> {
    delimited('"', take_till(0.., '"').map(|s: &str| s.to_string()), '"')
        .map(SExpr::String)
        .parse_next(input)
}

fn sexpr_list(input: &mut &str) -> PResult<SExpr> {
    delimited('(', repeat(0.., sexpr_value), ')')
        .map(SExpr::List)
        .parse_next(input)
}

// Configuration File Parser

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub entries: Vec<ConfigEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigEntry {
    pub key: String,
    pub value: ConfigValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Bool(bool),
    List(Vec<ConfigValue>),
}

pub fn parse_config(input: &str) -> Result<Config, String> {
    config_file.parse(input).map_err(|e| e.to_string())
}

fn config_file(input: &mut &str) -> PResult<Config> {
    repeat(0.., config_entry)
        .map(|entries| Config { entries })
        .parse_next(input)
}

fn config_entry(input: &mut &str) -> PResult<ConfigEntry> {
    config_ws(input)?;
    let key = config_key(input)?;
    config_ws(input)?;
    '='.parse_next(input)?;
    config_ws(input)?;
    let value = config_value(input)?;
    let _ = alt::<_, _, (), _>(('\n', '\r')).parse_next(input).ok();

    Ok(ConfigEntry { key, value })
}

fn config_ws(input: &mut &str) -> PResult<()> {
    take_while(0.., |c: char| c == ' ' || c == '\t')
        .void()
        .parse_next(input)
}

fn config_key(input: &mut &str) -> PResult<String> {
    (
        alpha1,
        take_while(0.., |c: char| {
            c.is_ascii_alphanumeric() || c == '_' || c == '.'
        }),
    )
        .take()
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

fn config_value(input: &mut &str) -> PResult<ConfigValue> {
    alt((
        "true".value(ConfigValue::Bool(true)),
        "false".value(ConfigValue::Bool(false)),
        config_number,
        config_string,
        config_list,
    ))
    .parse_next(input)
}

fn config_number(input: &mut &str) -> PResult<ConfigValue> {
    take_while(1.., |c: char| c.is_ascii_digit() || c == '.' || c == '-')
        .try_map(|s: &str| s.parse::<f64>().map(ConfigValue::Number))
        .parse_next(input)
}

fn config_string(input: &mut &str) -> PResult<ConfigValue> {
    delimited('"', take_till(0.., '"').map(|s: &str| s.to_string()), '"')
        .map(ConfigValue::String)
        .parse_next(input)
}

fn config_list(input: &mut &str) -> PResult<ConfigValue> {
    delimited(
        '[',
        delimited(
            config_ws,
            separated(0.., config_value, delimited(config_ws, ',', config_ws)),
            config_ws,
        ),
        ']',
    )
    .map(ConfigValue::List)
    .parse_next(input)
}

// URL Parser

#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

pub fn parse_url(input: &str) -> Result<Url, String> {
    url.parse(input).map_err(|e| e.to_string())
}

fn url(input: &mut &str) -> PResult<Url> {
    let scheme = terminated(alpha1, "://")
        .map(|s: &str| s.to_string())
        .parse_next(input)?;

    let host = take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || c == '.' || c == '-'
    })
    .map(|s: &str| s.to_string())
    .parse_next(input)?;

    let port = winnow::combinator::opt(preceded(':', digit1.try_map(|s: &str| s.parse::<u16>())))
        .parse_next(input)?;

    let path = winnow::combinator::opt(
        take_while(1.., |c: char| c != '?' && c != '#').map(|s: &str| s.to_string()),
    )
    .parse_next(input)?
    .unwrap_or_default();

    let query = winnow::combinator::opt(preceded(
        '?',
        take_while(1.., |c: char| c != '#').map(|s: &str| s.to_string()),
    ))
    .parse_next(input)?;

    let fragment = winnow::combinator::opt(preceded(
        '#',
        winnow::token::rest.map(|s: &str| s.to_string()),
    ))
    .parse_next(input)?;

    Ok(Url {
        scheme,
        host,
        port,
        path,
        query,
        fragment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        assert_eq!(parse_expression("42").unwrap(), Expr::Number(42.0));
        assert_eq!(parse_expression("3.14").unwrap(), Expr::Number(3.14));

        let expr = parse_expression("1 + 2").unwrap();
        assert_eq!(expr.eval(), 3.0);

        let expr = parse_expression("1 + 2 * 3").unwrap();
        assert_eq!(expr.eval(), 7.0);

        let expr = parse_expression("(1 + 2) * 3").unwrap();
        assert_eq!(expr.eval(), 9.0);

        let expr = parse_expression("10 - 5 / 2").unwrap();
        assert_eq!(expr.eval(), 7.5);
    }

    #[test]
    fn test_json() {
        assert_eq!(parse_json("null").unwrap(), Json::Null);
        assert_eq!(parse_json("true").unwrap(), Json::Bool(true));
        assert_eq!(parse_json("false").unwrap(), Json::Bool(false));
        assert_eq!(parse_json("42").unwrap(), Json::Number(42.0));
        assert_eq!(parse_json("3.14").unwrap(), Json::Number(3.14));
        assert_eq!(
            parse_json("\"hello\"").unwrap(),
            Json::String("hello".to_string())
        );

        assert_eq!(parse_json("[]").unwrap(), Json::Array(vec![]));

        assert_eq!(
            parse_json("[1, 2, 3]").unwrap(),
            Json::Array(vec![
                Json::Number(1.0),
                Json::Number(2.0),
                Json::Number(3.0)
            ])
        );

        assert_eq!(parse_json("{}").unwrap(), Json::Object(vec![]));

        assert_eq!(
            parse_json(r#"{"name": "Alice", "age": 30}"#).unwrap(),
            Json::Object(vec![
                ("name".to_string(), Json::String("Alice".to_string())),
                ("age".to_string(), Json::Number(30.0)),
            ])
        );

        let nested = r#"
        {
            "user": {
                "name": "Bob",
                "scores": [10, 20, 30]
            }
        }
        "#;
        let result = parse_json(nested).unwrap();
        match result {
            Json::Object(pairs) => {
                assert_eq!(pairs.len(), 1);
                assert_eq!(pairs[0].0, "user");
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_sexpr() {
        assert_eq!(parse_sexpr("42").unwrap(), SExpr::Number(42));
        assert_eq!(parse_sexpr("-10").unwrap(), SExpr::Number(-10));
        assert_eq!(
            parse_sexpr("foo").unwrap(),
            SExpr::Symbol("foo".to_string())
        );
        assert_eq!(
            parse_sexpr("\"hello\"").unwrap(),
            SExpr::String("hello".to_string())
        );

        assert_eq!(parse_sexpr("()").unwrap(), SExpr::List(vec![]));

        assert_eq!(
            parse_sexpr("(+ 1 2)").unwrap(),
            SExpr::List(vec![
                SExpr::Symbol("+".to_string()),
                SExpr::Number(1),
                SExpr::Number(2),
            ])
        );

        assert_eq!(
            parse_sexpr("(define (square x) (* x x))").unwrap(),
            SExpr::List(vec![
                SExpr::Symbol("define".to_string()),
                SExpr::List(vec![
                    SExpr::Symbol("square".to_string()),
                    SExpr::Symbol("x".to_string()),
                ]),
                SExpr::List(vec![
                    SExpr::Symbol("*".to_string()),
                    SExpr::Symbol("x".to_string()),
                    SExpr::Symbol("x".to_string()),
                ]),
            ])
        );
    }

    #[test]
    fn test_config() {
        let simple = "key = \"value\"\n";
        let result = parse_config(simple).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].key, "key");
        assert_eq!(
            result.entries[0].value,
            ConfigValue::String("value".to_string())
        );

        let multi = "name = \"Alice\"\nage = 30\nenabled = true\n";
        let result = parse_config(multi).unwrap();
        assert_eq!(result.entries.len(), 3);
        assert_eq!(
            result.entries[0].value,
            ConfigValue::String("Alice".to_string())
        );
        assert_eq!(result.entries[1].value, ConfigValue::Number(30.0));
        assert_eq!(result.entries[2].value, ConfigValue::Bool(true));

        let with_list = "servers = [\"web1\", \"web2\", \"web3\"]\n";
        let result = parse_config(with_list).unwrap();
        assert_eq!(result.entries.len(), 1);
        match &result.entries[0].value {
            ConfigValue::List(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_url() {
        let url = parse_url("http://example.com").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "");

        let url = parse_url("https://example.com:8080/path/to/resource").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/path/to/resource");

        let url = parse_url("http://example.com/search?q=rust&limit=10").unwrap();
        assert_eq!(url.path, "/search");
        assert_eq!(url.query, Some("q=rust&limit=10".to_string()));

        let url = parse_url("https://example.com/page#section").unwrap();
        assert_eq!(url.path, "/page");
        assert_eq!(url.fragment, Some("section".to_string()));
    }
}
