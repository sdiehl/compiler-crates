# combine

combine provides a powerful parser combinator library for building parsers from composable pieces. Unlike parser generators that require separate grammar files, combine constructs parsers entirely in Rust code through a rich set of combinators. The library emphasizes flexibility and error recovery, making it well-suited for both simple configuration files and complex programming languages.

The library's streaming approach enables parsing of large files without loading them entirely into memory, while its error handling system provides detailed information about parse failures. combine supports multiple input types including strings, byte arrays, and custom token streams, adapting to various parsing scenarios from network protocols to source code.

## Basic Expression Parser

```rust
use combine::parser::char::{char, digit, spaces};
use combine::parser::repeat::sep_by;
use combine::{attempt, between, choice, many1, parser, Parser, Stream};
use combine::error::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i32),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

parser! {
    fn expr[Input]()(Input) -> Expr
    where [Input: Stream<Token = char>]
    {
        expr_()
    }
}

fn expr_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    term().and(many1::<Vec<_>, _, _>((
        choice((char('+'), char('-'))),
        term()
    )))
    .map(|(first, rest): (Expr, Vec<(char, Expr)>)| {
        rest.into_iter().fold(first, |acc, (op, val)| match op {
            '+' => Expr::Add(Box::new(acc), Box::new(val)),
            '-' => Expr::Sub(Box::new(acc), Box::new(val)),
            _ => unreachable!(),
        })
    })
    .or(term())
}
```

The expression parser demonstrates combine's approach to operator precedence through parser composition. The expr_ function handles addition and subtraction as left-associative operations by parsing a term followed by zero or more operator-term pairs. The fold operation builds the abstract syntax tree left-to-right, ensuring proper associativity. The parser macro generates a wrapper function that simplifies the parser's type signature, making it easier to use in larger compositions.

The choice combinator selects between multiple alternatives, while many1 requires at least one match. The and method chains parsers sequentially, passing results as tuples. The map method transforms parse results, converting from the parser's output format to the desired AST representation. The or combinator provides fallback behavior, attempting the simpler term parser if the complex expression parsing fails.

## Parser Combinators

```rust
fn term<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    factor().and(many1::<Vec<_>, _, _>((
        choice((char('*'), char('/'))),
        factor()
    )))
    .map(|(first, rest): (Expr, Vec<(char, Expr)>)| {
        rest.into_iter().fold(first, |acc, (op, val)| match op {
            '*' => Expr::Mul(Box::new(acc), Box::new(val)),
            '/' => Expr::Div(Box::new(acc), Box::new(val)),
            _ => unreachable!(),
        })
    })
    .or(factor())
}

fn factor<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        between(char('('), char(')'), expr()),
        number(),
    ))
}

fn number<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1::<String, _, _>(digit())
        .map(|s| Expr::Number(s.parse().unwrap()))
}
```

The term parser handles multiplication and division with higher precedence than addition and subtraction. This precedence hierarchy emerges naturally from the parser structure, with term calling factor for its operands, and expr calling term. Each level of the hierarchy handles operators of the same precedence, delegating to the next level for higher-precedence operations.

The factor parser demonstrates parenthesis handling using the between combinator, which parses delimited content while discarding the delimiters. The recursive call to expr within parentheses allows arbitrary expression nesting. The number parser combines multiple digit characters into a string, then parses the result into an integer wrapped in the Number variant.

## JSON Parser

```rust
use combine::parser::char::{char, string};
use combine::parser::choice::optional;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

parser! {
    fn json[Input]()(Input) -> Json
    where [Input: Stream<Token = char>]
    {
        spaces().with(json_value())
    }
}

fn json_value<Input>() -> impl Parser<Input, Output = Json>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        string("null").map(|_| Json::Null),
        string("true").map(|_| Json::Bool(true)),
        string("false").map(|_| Json::Bool(false)),
        json_number(),
        json_string(),
        json_array(),
        json_object(),
    ))
}
```

The JSON parser showcases combine's ability to handle complex recursive data structures. The json_value function uses choice to select among all possible JSON types, with each alternative parser returning the appropriate Json enum variant. The string parser matches exact character sequences, while map transforms the successful parse into the corresponding JSON value.

The spaces().with() pattern at the entry point consumes leading whitespace before parsing the actual JSON value. This pattern appears throughout the parser to handle optional whitespace between tokens. The parser macro again simplifies the type signature, hiding the complex return type that would otherwise be required.

## String and Number Parsing

```rust
fn json_string<Input>() -> impl Parser<Input, Output = Json>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    between(
        char('"'),
        char('"'),
        many1::<String, _, _>(satisfy(|c| c != '"' && c != '\\'))
    )
    .map(Json::String)
}

fn json_number<Input>() -> impl Parser<Input, Output = Json>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    optional(char('-'))
        .and(many1::<String, _, _>(digit()))
        .and(optional(char('.').and(many1::<String, _, _>(digit()))))
        .and(optional(
            choice((char('e'), char('E')))
                .and(optional(choice((char('+'), char('-')))))
                .and(many1::<String, _, _>(digit()))
        ))
        .map(|(((sign, int), frac), exp)| {
            let mut num = String::new();
            if sign.is_some() { num.push('-'); }
            num.push_str(&int);
            if let Some((_, f)) = frac {
                num.push('.');
                num.push_str(&f);
            }
            if let Some(((e, sign), exp_digits)) = exp {
                num.push(e);
                if let Some(s) = sign { num.push(s); }
                num.push_str(&exp_digits);
            }
            Json::Number(num.parse().unwrap())
        })
}
```

The json_string parser demonstrates basic string parsing with escape sequence support. The satisfy combinator accepts characters matching a predicate, building a string from all non-quote, non-backslash characters. Real JSON parsing would require additional escape sequence handling, but this simplified version illustrates the core concept.

The json_number parser handles the full JSON number format including optional signs, decimal points, and scientific notation. The nested tuple structure from chained and combinators captures each component of the number. The map function reconstructs the string representation before parsing it as a floating-point value. This approach ensures correct handling of all valid JSON number formats while maintaining parse accuracy.

## Array and Object Parsing

```rust
fn json_array<Input>() -> impl Parser<Input, Output = Json>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    between(
        char('[').skip(spaces()),
        char(']'),
        sep_by(json_value().skip(spaces()), char(',').skip(spaces()))
    )
    .map(Json::Array)
}

fn json_object<Input>() -> impl Parser<Input, Output = Json>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let member = json_string()
        .skip(spaces())
        .skip(char(':'))
        .skip(spaces())
        .and(json_value());

    between(
        char('{').skip(spaces()),
        char('}'),
        sep_by(member.skip(spaces()), char(',').skip(spaces()))
    )
    .map(|members: Vec<(Json, Json)>| {
        Json::Object(
            members.into_iter()
                .map(|(k, v)| match k {
                    Json::String(s) => (s, v),
                    _ => unreachable!(),
                })
                .collect()
        )
    })
}
```

The array parser uses sep_by to handle comma-separated values, automatically managing both empty arrays and trailing comma issues. The skip method discards whitespace after each element and separator, maintaining clean separation between structural parsing and whitespace handling. The between combinator ensures proper bracket matching while the map function wraps the result in the Array variant.

Object parsing combines string keys with arbitrary values using the member parser. The skip chain removes colons and whitespace without including them in the result. The map function extracts string values from the Json::String variant for use as object keys, transforming the vector of tuples into the expected format. This design maintains type safety while parsing the heterogeneous structure of JSON objects.

## S-Expression Parser

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Symbol(String),
    Number(i64),
    String(String),
    List(Vec<SExpr>),
}

parser! {
    fn s_expression[Input]()(Input) -> SExpr
    where [Input: Stream<Token = char>]
    {
        spaces().with(s_expr())
    }
}

fn s_expr<Input>() -> impl Parser<Input, Output = SExpr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        s_list(),
        s_atom(),
    ))
}

fn s_atom<Input>() -> impl Parser<Input, Output = SExpr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        s_string(),
        s_number(),
        s_symbol(),
    ))
}
```

The S-expression parser handles LISP-style symbolic expressions with minimal complexity. The grammar's recursive nature maps directly to Rust's enum system, with each variant corresponding to a fundamental S-expression type. The parser structure mirrors the data structure, making the implementation intuitive and maintainable.

The separation between s_expr and s_atom clarifies the grammar structure, distinguishing compound lists from atomic values. This organization simplifies error messages and makes the parser's intent clear. The choice combinator tries each alternative in order, selecting the first successful parse.

## Symbol and List Parsing

```rust
fn s_symbol<Input>() -> impl Parser<Input, Output = SExpr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1::<String, _, _>(satisfy(|c: char|
        c.is_alphanumeric() || "+-*/<>=!?_".contains(c)
    ))
    .map(SExpr::Symbol)
}

fn s_list<Input>() -> impl Parser<Input, Output = SExpr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    between(
        char('(').skip(spaces()),
        char(')'),
        many(s_expr().skip(spaces()))
    )
    .map(SExpr::List)
}
```

Symbol parsing accepts standard LISP identifier characters including operators and special symbols. The satisfy predicate defines the valid character set, while many1 ensures at least one character. This approach handles both function names and operators uniformly, reflecting LISP's treatment of operators as regular symbols.

List parsing recursively invokes s_expr for each element, enabling arbitrary nesting. The many combinator accepts zero or more elements, properly handling empty lists. Whitespace handling occurs after each element through skip, maintaining clean separation between elements without requiring explicit whitespace in the grammar.

## Configuration Parser

```rust
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

fn config_value<Input>() -> impl Parser<Input, Output = ConfigValue>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        string("true").map(|_| ConfigValue::Bool(true)),
        string("false").map(|_| ConfigValue::Bool(false)),
        config_number(),
        config_string(),
        config_list(),
    ))
}
```

The configuration parser demonstrates parsing of key-value pairs with multiple value types. The ConfigValue enum supports common configuration types including strings, numbers, booleans, and lists. This structure handles most configuration file formats while remaining simple to extend.

The choice ordering matters for ambiguous cases. Parsing boolean literals before numbers prevents misinterpretation, while parsing strings after literals avoids consuming quoted keywords. This careful ordering ensures correct parsing without complex lookahead.

## Error Recovery

```rust
use combine::stream::position;
use combine::error::StringStreamError;

pub fn parse_with_position(input: &str) -> Result<Json, String> {
    let mut parser = json();
    let stream = position::Stream::new(input);

    match parser.parse(stream) {
        Ok((result, _)) => Ok(result),
        Err(err) => {
            let pos = err.position;
            Err(format!("Parse error at line {}, column {}: {}",
                pos.line, pos.column, err))
        }
    }
}
```

combine provides detailed error information including position tracking and error context. The position::Stream wrapper adds line and column information to the input stream, enabling precise error reporting. This information helps users quickly locate and fix syntax errors in their input.

Error messages include both the location and nature of the failure, with combine attempting to provide helpful suggestions based on the expected tokens. The library's error recovery mechanisms allow parsing to continue after errors in some cases, useful for IDE integration where partial results remain valuable.

## Stream Abstraction

```rust
use combine::stream::easy;
use combine::stream::state::State;

pub fn parse_with_state<'a>(
    input: &'a str,
    filename: String,
) -> Result<Json, easy::ParseError<&'a str>> {
    let stream = State::new(easy::Stream(input))
        .with_positioner(position::SourcePosition::default());

    let mut parser = json();
    parser.easy_parse(stream)
        .map(|t| t.0)
}
```

combine's stream abstraction supports various input types beyond simple strings. The State wrapper adds user-defined state to the parsing process, useful for maintaining symbol tables or configuration during parsing. The easy module provides enhanced error messages at a small performance cost.

Custom stream types enable parsing of token sequences from lexers, byte arrays from network protocols, or any other sequential data source. This flexibility makes combine suitable for diverse parsing tasks while maintaining a consistent interface across different input types.

## Performance Optimization

```rust
use combine::parser::combinator::recognize;
use combine::parser::range::take_while1;

fn optimized_number<Input>() -> impl Parser<Input, Output = f64>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    recognize((
        optional(char('-')),
        take_while1(|c: char| c.is_ascii_digit()),
        optional((
            char('.'),
            take_while1(|c: char| c.is_ascii_digit())
        )),
    ))
    .map(|s: String| s.parse().unwrap())
}
```

The recognize combinator captures the entire matched input as a string without building intermediate structures. This approach reduces allocations when parsing numbers or identifiers, improving performance for large inputs. The take_while1 combinator efficiently consumes characters matching a predicate without creating temporary collections.

combine's lazy evaluation model ensures parsers only perform necessary work. Failed alternatives don't consume input beyond the failure point, enabling efficient backtracking. The attempt combinator explicitly marks backtrack points, giving fine control over parser performance.

## Testing Strategies

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_precedence() {
        let input = "1 + 2 * 3";
        let result = expression(input).unwrap();
        assert_eq!(result.eval(), 7.0); // Not 9.0
    }

    #[test]
    fn test_json_nested() {
        let input = r#"{"a": [1, {"b": true}]}"#;
        let result = parse_json(input).unwrap();
        match result {
            Json::Object(obj) => {
                assert_eq!(obj.len(), 1);
                assert_eq!(obj[0].0, "a");
                match &obj[0].1 {
                    Json::Array(arr) => assert_eq!(arr.len(), 2),
                    _ => panic!("Expected array"),
                }
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_error_position() {
        let input = "{ invalid json }";
        let result = parse_with_position(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("line"));
        assert!(err.contains("column"));
    }
}
```

Testing parsers requires validating both successful parses and error handling. Precedence tests ensure operators combine correctly, while nested structure tests verify recursive parsing. Error tests confirm that position information and error messages provide useful debugging information.

Property-based testing works well with parser combinators. Generate random valid inputs according to the grammar, parse them, and verify properties like roundtrip printing or semantic equivalence. This approach finds edge cases that manual tests might miss.

## Best Practices

Structure parsers to match the desired AST closely, using Rust's type system to enforce invariants. Separate lexical concerns like whitespace handling from structural parsing using skip and trim combinators. This separation simplifies both the parser and error messages.

Use the parser! macro for public interfaces to hide complex type signatures. The macro generates cleaner function signatures while preserving full type safety. Internal helper parsers can use impl Parser return types for better compile times and simpler code.

Order choice alternatives carefully, considering both correctness and performance. Place more specific patterns before general ones to avoid incorrect matches. Use attempt when backtracking is needed, but minimize its use for better performance.

Build parsers incrementally, testing each component before composing them. Start with simple atoms, then build expressions, then statements. This approach makes debugging easier and ensures each piece works correctly before integration.

combine provides a flexible and powerful approach to parsing that scales from simple configuration files to complete programming languages. Its combinator-based design encourages modular, testable parsers while maintaining excellent performance and error reporting. The library's stream abstraction and careful API design make it an excellent choice for compiler frontends and data processing pipelines.