//! combine - Parser combinator library with streaming support and excellent
//! error messages

use std::collections::HashMap;

use combine::parser::char::{char, digit, letter, spaces, string};
use combine::parser::choice::choice;
use combine::parser::repeat::{many, many1, sep_by};
use combine::parser::sequence::between;
use combine::{eof, optional, parser, satisfy, Parser, Stream};

/// AST types for arithmetic expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Var(String),
}

/// Parse arithmetic expressions with operator precedence
pub fn expression<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>, {
    spaces().with(expr())
}

parser! {
    fn expr[Input]()(Input) -> Expr
    where [Input: Stream<Token = char>]
    {
        let op = choice((char('+'), char('-')));

        term().skip(spaces()).and(many((op.skip(spaces()), term().skip(spaces())))).map(
            |(first, rest): (Expr, Vec<(char, Expr)>)| {
                rest.into_iter().fold(first, |acc, (op, val)| match op {
                    '+' => Expr::Add(Box::new(acc), Box::new(val)),
                    '-' => Expr::Sub(Box::new(acc), Box::new(val)),
                    _ => unreachable!(),
                })
            },
        )
    }
}

parser! {
    fn term[Input]()(Input) -> Expr
    where [Input: Stream<Token = char>]
    {
        let op = choice((char('*'), char('/')));

        factor().skip(spaces()).and(many((op.skip(spaces()), factor().skip(spaces())))).map(
            |(first, rest): (Expr, Vec<(char, Expr)>)| {
                rest.into_iter().fold(first, |acc, (op, val)| match op {
                    '*' => Expr::Mul(Box::new(acc), Box::new(val)),
                    '/' => Expr::Div(Box::new(acc), Box::new(val)),
                    _ => unreachable!(),
                })
            },
        )
    }
}

parser! {
    fn factor[Input]()(Input) -> Expr
    where [Input: Stream<Token = char>]
    {
        choice((
            number(),
            identifier().map(Expr::Var),
            char('-').with(factor()).map(|e| Expr::Neg(Box::new(e))),
            between(char('('), char(')'), spaces().with(expr())),
        ))
    }
}

fn number<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>, {
    let integer = many1(digit());
    let decimal = optional(char('.').with(many(digit())));

    (integer, decimal).map(|(int, dec): (String, Option<String>)| {
        let num = if let Some(dec) = dec {
            format!("{}.{}", int, dec)
        } else {
            int
        };
        Expr::Number(num.parse().unwrap())
    })
}

fn identifier<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>, {
    (letter(), many(choice((letter(), digit(), char('_')))))
        .map(|(first, rest): (char, String)| format!("{}{}", first, rest))
}

impl Expr {
    /// Evaluate expression with variable bindings
    pub fn eval(&self, vars: &HashMap<String, f64>) -> Result<f64, String> {
        match self {
            Expr::Number(n) => Ok(*n),
            Expr::Add(l, r) => Ok(l.eval(vars)? + r.eval(vars)?),
            Expr::Sub(l, r) => Ok(l.eval(vars)? - r.eval(vars)?),
            Expr::Mul(l, r) => Ok(l.eval(vars)? * r.eval(vars)?),
            Expr::Div(l, r) => {
                let right = r.eval(vars)?;
                if right == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(l.eval(vars)? / right)
                }
            }
            Expr::Neg(e) => Ok(-e.eval(vars)?),
            Expr::Var(name) => vars
                .get(name)
                .copied()
                .ok_or_else(|| format!("Undefined variable: {}", name)),
        }
    }
}

/// JSON value type
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

parser! {
    pub fn json_value[Input]()(Input) -> JsonValue
    where [Input: Stream<Token = char>]
    {
        spaces().with(choice((
            string("null").map(|_| JsonValue::Null),
            string("true").map(|_| JsonValue::Bool(true)),
            string("false").map(|_| JsonValue::Bool(false)),
            json_number(),
            json_string(),
            json_array(),
            json_object(),
        )))
    }
}

fn json_number<Input>() -> impl Parser<Input, Output = JsonValue>
where
    Input: Stream<Token = char>, {
    let sign = optional(char('-'));
    let integer = many1::<String, _, _>(digit());
    let decimal = optional(char('.').with(many1::<String, _, _>(digit())));
    let exponent = optional(
        choice((char('e'), char('E')))
            .with(optional(choice((char('+'), char('-')))))
            .and(many1::<String, _, _>(digit())),
    );

    (sign, integer, decimal, exponent).map(|(sign, int, dec, exp)| {
        let mut num = String::new();
        if sign.is_some() {
            num.push('-');
        }
        num.push_str(&int);
        if let Some(dec) = dec {
            num.push('.');
            num.push_str(&dec);
        }
        if let Some((exp_sign, exp_val)) = exp {
            num.push('e');
            if let Some(s) = exp_sign {
                num.push(s);
            }
            num.push_str(&exp_val);
        }
        JsonValue::Number(num.parse().unwrap())
    })
}

fn json_string<Input>() -> impl Parser<Input, Output = JsonValue>
where
    Input: Stream<Token = char>, {
    between(
        char('"'),
        char('"'),
        many(choice((
            satisfy(|c: char| c != '"' && c != '\\'),
            char('\\').with(choice((
                char('"'),
                char('\\'),
                char('/'),
                char('b').map(|_| '\u{0008}'),
                char('f').map(|_| '\u{000C}'),
                char('n').map(|_| '\n'),
                char('r').map(|_| '\r'),
                char('t').map(|_| '\t'),
            ))),
        ))),
    )
    .map(|s: String| JsonValue::String(s))
}

parser! {
    fn json_array[Input]()(Input) -> JsonValue
    where [Input: Stream<Token = char>]
    {
        between(
            char('[').skip(spaces()),
            spaces().with(char(']')),
            sep_by(json_value(), spaces().with(char(',')).skip(spaces())),
        )
        .map(JsonValue::Array)
    }
}

parser! {
    fn json_object[Input]()(Input) -> JsonValue
    where [Input: Stream<Token = char>]
    {
        let pair = (
            json_string(),
            spaces().with(char(':')).skip(spaces()),
            json_value(),
        )
            .map(|(key, _, value)| {
                if let JsonValue::String(k) = key {
                    (k, value)
                } else {
                    unreachable!()
                }
            });

        between(
            char('{').skip(spaces()),
            spaces().with(char('}')),
            sep_by(pair, spaces().with(char(',')).skip(spaces())),
        )
        .map(|pairs: Vec<(String, JsonValue)>| JsonValue::Object(pairs.into_iter().collect()))
    }
}

/// S-expression type
#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Symbol(String),
    Number(i64),
    String(String),
    List(Vec<SExpr>),
}

/// S-expression parser
pub fn s_expression<Input>() -> impl Parser<Input, Output = SExpr>
where
    Input: Stream<Token = char>, {
    spaces().with(s_expr())
}

parser! {
    fn s_expr[Input]()(Input) -> SExpr
    where [Input: Stream<Token = char>]
    {
        choice((s_list(), s_string(), s_number(), s_symbol()))
    }
}

parser! {
    fn s_list[Input]()(Input) -> SExpr
    where [Input: Stream<Token = char>]
    {
        between(
            char('(').skip(spaces()),
            spaces().with(char(')')),
            sep_by(s_expr(), spaces()),
        )
        .map(SExpr::List)
    }
}

fn s_symbol<Input>() -> impl Parser<Input, Output = SExpr>
where
    Input: Stream<Token = char>, {
    many1(satisfy(|c: char| {
        c.is_alphanumeric() || "+-*/_<>=!?".contains(c)
    }))
    .map(SExpr::Symbol)
}

fn s_number<Input>() -> impl Parser<Input, Output = SExpr>
where
    Input: Stream<Token = char>, {
    let sign = optional(char('-'));
    let digits = many1(digit());

    (sign, digits).map(|(sign, num): (Option<char>, String)| {
        let n = if sign.is_some() {
            format!("-{}", num)
        } else {
            num
        };
        SExpr::Number(n.parse().unwrap())
    })
}

fn s_string<Input>() -> impl Parser<Input, Output = SExpr>
where
    Input: Stream<Token = char>, {
    between(char('"'), char('"'), many(satisfy(|c: char| c != '"'))).map(SExpr::String)
}

/// Configuration language AST
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

/// Parse configuration file
pub fn config<Input>() -> impl Parser<Input, Output = Config>
where
    Input: Stream<Token = char>, {
    spaces()
        .with(many(config_entry().skip(spaces())))
        .skip(eof())
        .map(|entries| Config { entries })
}

fn config_entry<Input>() -> impl Parser<Input, Output = ConfigEntry>
where
    Input: Stream<Token = char>, {
    let key = many1(satisfy(|c: char| {
        c.is_alphanumeric() || c == '_' || c == '.'
    }));
    let eq = spaces().with(char('=')).skip(spaces());

    (key, eq, config_value()).map(|(key, _, value)| ConfigEntry { key, value })
}

parser! {
    fn config_value[Input]()(Input) -> ConfigValue
    where [Input: Stream<Token = char>]
    {
        choice((
            config_string(),
            config_bool(),
            config_number(),
            config_list(),
        ))
    }
}

fn config_string<Input>() -> impl Parser<Input, Output = ConfigValue>
where
    Input: Stream<Token = char>, {
    between(char('"'), char('"'), many(satisfy(|c: char| c != '"'))).map(ConfigValue::String)
}

fn config_bool<Input>() -> impl Parser<Input, Output = ConfigValue>
where
    Input: Stream<Token = char>, {
    choice((
        string("true").map(|_| ConfigValue::Bool(true)),
        string("false").map(|_| ConfigValue::Bool(false)),
    ))
}

fn config_number<Input>() -> impl Parser<Input, Output = ConfigValue>
where
    Input: Stream<Token = char>, {
    let sign = optional(char('-'));
    let integer = many1(digit());
    let decimal = optional(char('.').with(many1(digit())));

    (sign, integer, decimal).map(|(sign, int, dec): (Option<char>, String, Option<String>)| {
        let mut num = String::new();
        if sign.is_some() {
            num.push('-');
        }
        num.push_str(&int);
        if let Some(dec) = dec {
            num.push('.');
            num.push_str(&dec);
        }
        ConfigValue::Number(num.parse().unwrap())
    })
}

parser! {
    fn config_list[Input]()(Input) -> ConfigValue
    where [Input: Stream<Token = char>]
    {
        between(
            char('[').skip(spaces()),
            spaces().with(char(']')),
            sep_by(config_value(), spaces().with(char(',')).skip(spaces())),
        )
        .map(ConfigValue::List)
    }
}

#[cfg(test)]
mod tests {
    use combine::EasyParser;

    use super::*;

    #[test]
    fn test_expression_parsing() {
        let result = expression().easy_parse("2 + 3 * 4");
        assert!(result.is_ok());
        let (expr, _) = result.unwrap();
        let vars = HashMap::new();
        assert_eq!(expr.eval(&vars).unwrap(), 14.0); // 2 + (3 * 4)

        let result = expression().easy_parse("(2 + 3) * 4");
        assert!(result.is_ok());
        let (expr, _) = result.unwrap();
        assert_eq!(expr.eval(&vars).unwrap(), 20.0); // (2 + 3) * 4

        let result = expression().easy_parse("-5 + 10");
        assert!(result.is_ok());
        let (expr, _) = result.unwrap();
        assert_eq!(expr.eval(&vars).unwrap(), 5.0);

        // With variables
        let result = expression().easy_parse("x * 2 + y");
        assert!(result.is_ok());
        let (expr, _) = result.unwrap();
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 5.0);
        vars.insert("y".to_string(), 3.0);
        assert_eq!(expr.eval(&vars).unwrap(), 13.0);
    }

    #[test]
    fn test_json_parsing() {
        let result = json_value().easy_parse("null");
        assert_eq!(result, Ok((JsonValue::Null, "")));

        let result = json_value().easy_parse("true");
        assert_eq!(result, Ok((JsonValue::Bool(true), "")));

        let result = json_value().easy_parse("42.5");
        assert_eq!(result, Ok((JsonValue::Number(42.5), "")));

        let result = json_value().easy_parse(r#""hello world""#);
        assert_eq!(
            result,
            Ok((JsonValue::String("hello world".to_string()), ""))
        );

        let result = json_value().easy_parse("[1, 2, 3]");
        assert_eq!(
            result,
            Ok((
                JsonValue::Array(vec![
                    JsonValue::Number(1.0),
                    JsonValue::Number(2.0),
                    JsonValue::Number(3.0)
                ]),
                ""
            ))
        );

        let result = json_value().easy_parse(r#"{"name": "John", "age": 30}"#);
        assert!(result.is_ok());
        if let (JsonValue::Object(map), _) = result.unwrap() {
            assert_eq!(
                map.get("name"),
                Some(&JsonValue::String("John".to_string()))
            );
            assert_eq!(map.get("age"), Some(&JsonValue::Number(30.0)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_s_expression_parsing() {
        let result = s_expression().easy_parse("42");
        assert_eq!(result, Ok((SExpr::Number(42), "")));

        let result = s_expression().easy_parse("hello");
        assert_eq!(result, Ok((SExpr::Symbol("hello".to_string()), "")));

        let result = s_expression().easy_parse("(+ 1 2)");
        assert_eq!(
            result,
            Ok((
                SExpr::List(vec![
                    SExpr::Symbol("+".to_string()),
                    SExpr::Number(1),
                    SExpr::Number(2)
                ]),
                ""
            ))
        );

        let result = s_expression().easy_parse("(define (square x) (* x x))");
        assert!(result.is_ok());
        if let (SExpr::List(items), _) = result.unwrap() {
            assert_eq!(items[0], SExpr::Symbol("define".to_string()));
            assert!(matches!(items[1], SExpr::List(_)));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_config_parsing() {
        let input = r#"
            host = "localhost"
            port = 8080
            debug = true
            features = ["auth", "logging"]
        "#;

        let result = config().easy_parse(input);
        assert!(result.is_ok());
        let (cfg, _) = result.unwrap();
        assert_eq!(cfg.entries.len(), 4);
        assert_eq!(cfg.entries[0].key, "host");
        assert_eq!(
            cfg.entries[0].value,
            ConfigValue::String("localhost".to_string())
        );
        assert_eq!(cfg.entries[1].key, "port");
        assert_eq!(cfg.entries[1].value, ConfigValue::Number(8080.0));
        assert_eq!(cfg.entries[2].key, "debug");
        assert_eq!(cfg.entries[2].value, ConfigValue::Bool(true));
    }

    #[test]
    fn test_error_recovery() {
        let result = expression().easy_parse("2 + + 3");
        assert!(result.is_err());

        let result = json_value().easy_parse("{invalid}");
        assert!(result.is_err());

        let result = s_expression().easy_parse("(unclosed");
        assert!(result.is_err());
    }
}
