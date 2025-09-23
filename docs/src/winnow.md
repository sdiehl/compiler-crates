# winnow

winnow is a parser combinator library that emphasizes performance, ergonomics, and flexibility. Building on lessons learned from nom and other parser libraries, winnow provides a streamlined API for constructing parsers from simple, composable pieces. The library uses a mutable reference approach that enables better error messages and more intuitive parser composition while maintaining excellent performance characteristics.

The library's design philosophy centers on making the common case easy while keeping the complex possible. winnow parsers operate on mutable string slices, automatically advancing the input position as parsing proceeds. This approach eliminates the manual input management required by other libraries while providing clear semantics for parser composition and error handling.

## Arithmetic Expression Parser

```rust
use winnow::ascii::{digit1, space0};
use winnow::combinator::{alt, delimited, preceded, repeat};
use winnow::{PResult, Parser};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Paren(Box<Expr>),
}

pub fn parse_expression(input: &str) -> Result<Expr, String> {
    expr.parse(input).map_err(|e| e.to_string())
}

fn expr(input: &mut &str) -> PResult<Expr> {
    add_sub(input)
}

fn add_sub(input: &mut &str) -> PResult<Expr> {
    let init = mul_div(input)?;

    repeat(0.., (
        delimited(space0, alt(('+', '-')), space0),
        mul_div
    ))
    .fold(move || init.clone(), |acc, (op, val)| {
        match op {
            '+' => Expr::Add(Box::new(acc), Box::new(val)),
            '-' => Expr::Sub(Box::new(acc), Box::new(val)),
            _ => unreachable!(),
        }
    })
    .parse_next(input)
}
```

The expression parser demonstrates winnow's approach to building precedence-aware parsers. The mutable reference parameter automatically tracks position in the input, eliminating the need to explicitly thread state through the parser. The fold combinator builds left-associative operations by accumulating results, transforming a sequence of operations into a properly structured AST.

The PResult type alias simplifies error handling while maintaining full error information. The parse_next method advances the input reference, consuming matched characters. This mutation-based approach provides cleaner composition semantics than returning remaining input, as each parser clearly consumes its portion of the input.

## Factor and Number Parsing

```rust
fn mul_div(input: &mut &str) -> PResult<Expr> {
    let init = factor(input)?;

    repeat(0.., (
        delimited(space0, alt(('*', '/')), space0),
        factor
    ))
    .fold(move || init.clone(), |acc, (op, val)| {
        match op {
            '*' => Expr::Mul(Box::new(acc), Box::new(val)),
            '/' => Expr::Div(Box::new(acc), Box::new(val)),
            _ => unreachable!(),
        }
    })
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
```

The factor parser handles both atomic numbers and parenthesized expressions, demonstrating recursive parser composition. The alt combinator tries alternatives in order, selecting the first successful match. The delimited combinator parses bracketed content while the preceded combinator handles leading whitespace, showing how winnow's combinators compose naturally.

Number parsing uses take_while to consume digit characters and decimal points, then try_map to parse the string into a floating-point value. The try_map combinator propagates parsing errors properly, converting string parse failures into parser errors with appropriate context. This error handling ensures that invalid numbers produce meaningful error messages rather than panics.

## JSON Parser

```rust
use winnow::token::take_till;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

fn json_value(input: &mut &str) -> PResult<Json> {
    delimited(
        space0,
        alt((
            "null".value(Json::Null),
            "true".value(Json::Bool(true)),
            "false".value(Json::Bool(false)),
            json_number,
            json_string.map(Json::String),
            json_array,
            json_object,
        )),
        space0,
    )
    .parse_next(input)
}

fn json_string(input: &mut &str) -> PResult<String> {
    delimited(
        '"',
        take_till(0.., '"').map(|s: &str| s.to_string()),
        '"',
    )
    .parse_next(input)
}
```

The JSON parser showcases winnow's elegant handling of heterogeneous data structures. The value method on string literals creates parsers that return constant values upon matching, eliminating boilerplate mapping functions. This approach makes literal parsing concise while maintaining type safety.

String parsing uses take_till to consume characters until encountering a delimiter. This combinator efficiently handles variable-length content without backtracking. The simplified string parser shown here would need escape sequence handling for production use, but demonstrates the core parsing approach.

## Array and Object Parsing

```rust
use winnow::combinator::{separated, terminated};

fn json_array(input: &mut &str) -> PResult<Json> {
    delimited(
        '[',
        delimited(
            space0,
            separated(0.., json_value, delimited(space0, ',', space0)),
            space0,
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
            space0,
            separated(0.., json_member, delimited(space0, ',', space0)),
            space0,
        ),
        '}',
    )
    .map(Json::Object)
    .parse_next(input)
}

fn json_member(input: &mut &str) -> PResult<(String, Json)> {
    (
        terminated(json_string, delimited(space0, ':', space0)),
        json_value,
    )
        .parse_next(input)
}
```

Array parsing demonstrates winnow's separated combinator, which handles delimiter-separated lists with proper edge case handling. The nested delimited calls manage both the array brackets and internal whitespace, showing how combinators compose to handle complex formatting requirements. The 0.. range allows empty arrays while separated automatically handles trailing delimiter issues.

Object parsing combines multiple concepts including tuple parsing for key-value pairs. The terminated combinator discards the colon separator after parsing the key, while maintaining the key value. This design keeps parsing logic clear while building the exact data structure needed. The member parser returns tuples that map directly to the Object variant's expected type.

## S-Expression Parser

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Symbol(String),
    Number(i64),
    String(String),
    List(Vec<SExpr>),
}

fn sexpr_value(input: &mut &str) -> PResult<SExpr> {
    delimited(
        sexpr_ws,
        alt((
            sexpr_number,
            sexpr_string,
            sexpr_symbol,
            sexpr_list,
        )),
        sexpr_ws,
    )
    .parse_next(input)
}

fn sexpr_symbol(input: &mut &str) -> PResult<SExpr> {
    take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() ||
        c == '_' || c == '-' || c == '+' ||
        c == '*' || c == '/' || c == '?'
    })
    .map(|s: &str| SExpr::Symbol(s.to_string()))
    .parse_next(input)
}

fn sexpr_list(input: &mut &str) -> PResult<SExpr> {
    delimited(
        '(',
        repeat(0.., sexpr_value),
        ')',
    )
    .map(SExpr::List)
    .parse_next(input)
}
```

The S-expression parser illustrates recursive data structure parsing with minimal complexity. Symbol parsing accepts standard LISP identifier characters, including operators that are treated as regular symbols in LISP syntax. The take_while combinator with a minimum count ensures symbols contain at least one character while accepting any valid symbol character.

List parsing recursively calls sexpr_value for each element, enabling arbitrary nesting depth. The repeat combinator with 0.. accepts empty lists, matching LISP's treatment of () as a valid empty list. The clean separation between value parsing and whitespace handling keeps the grammar clear and maintainable.

## Configuration Parser

```rust
use winnow::ascii::{alpha1, alphanumeric1};

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub entries: Vec<ConfigEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigEntry {
    pub key: String,
    pub value: ConfigValue,
}

fn config_entry(input: &mut &str) -> PResult<ConfigEntry> {
    let _ = config_ws(input)?;
    let key = config_key(input)?;
    let _ = config_ws(input)?;
    let _ = '='.parse_next(input)?;
    let _ = config_ws(input)?;
    let value = config_value(input)?;
    let _ = alt(('\n', '\r')).parse_next(input).ok();

    Ok(ConfigEntry { key, value })
}

fn config_key(input: &mut &str) -> PResult<String> {
    (
        alpha1,
        take_while(0.., |c: char|
            c.is_ascii_alphanumeric() || c == '_' || c == '.'
        ),
    )
        .recognize()
        .map(|s: &str| s.to_string())
        .parse_next(input)
}
```

Configuration parsing demonstrates line-oriented parsing with winnow's explicit control flow. The config_entry function manually sequences parsing steps, providing clear control over whitespace handling and error recovery. This explicit approach makes the parser's behavior transparent, especially useful for formats where line boundaries matter.

The key parser uses recognize to capture the entire matched region as a string, avoiding the need to reconstruct the key from components. The tuple parser ensures keys start with a letter while allowing numbers and underscores in subsequent positions, enforcing common configuration file conventions.

## URL Parser

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

fn url(input: &mut &str) -> PResult<Url> {
    let scheme = terminated(alpha1, "://")
        .map(|s: &str| s.to_string())
        .parse_next(input)?;

    let host = take_while(1.., |c: char|
        c.is_ascii_alphanumeric() || c == '.' || c == '-'
    )
        .map(|s: &str| s.to_string())
        .parse_next(input)?;

    let port = winnow::combinator::opt(
        preceded(':', digit1.try_map(|s: &str| s.parse::<u16>()))
    )
        .parse_next(input)?;

    let path = alt((
        take_while(1.., |c: char| c != '?' && c != '#')
            .map(|s: &str| s.to_string()),
        winnow::combinator::empty.value(String::from("/")),
    ))
        .parse_next(input)?;

    let query = winnow::combinator::opt(
        preceded('?', take_while(1.., |c: char| c != '#')
            .map(|s: &str| s.to_string()))
    )
        .parse_next(input)?;

    let fragment = winnow::combinator::opt(
        preceded('#', winnow::combinator::rest
            .map(|s: &str| s.to_string()))
    )
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
```

URL parsing showcases sequential parsing with optional components. The scheme parser uses terminated to consume the :// separator while keeping only the scheme name. The opt combinator handles optional components like ports and query strings, returning None when the component is absent rather than failing the parse.

The path parser demonstrates fallback behavior using alt with empty, providing a default value when no path is specified. The rest combinator consumes all remaining input for the fragment, appropriate since fragments appear last in URLs. This structured approach handles the various optional components of URLs while maintaining parse correctness.

## Error Context

```rust
use winnow::error::ContextError;

fn number_with_context(input: &mut &str) -> PResult<f64> {
    take_while(1.., |c: char| c.is_ascii_digit() || c == '.')
        .try_map(|s: &str| s.parse::<f64>())
        .context("number")
        .parse_next(input)
}

fn json_value_with_context(input: &mut &str) -> PResult<Json> {
    delimited(
        space0,
        alt((
            "null".value(Json::Null).context("null"),
            "true".value(Json::Bool(true)).context("boolean"),
            "false".value(Json::Bool(false)).context("boolean"),
            number_with_context.map(Json::Number),
            json_string.map(Json::String).context("string"),
            json_array.context("array"),
            json_object.context("object"),
        )),
        space0,
    )
    .parse_next(input)
}
```

winnow's context system provides meaningful error messages by labeling parser components. The context method attaches a description to a parser, which appears in error messages when parsing fails. This labeling helps users understand what the parser expected at each position, crucial for debugging complex grammars.

Context annotations work throughout the parser hierarchy, with inner contexts providing specific details while outer contexts give broader structure. This layered approach produces error messages that guide users from high-level structure down to specific token requirements.

## Stream Positioning

```rust
use winnow::stream::Checkpoint;

fn parse_with_recovery(input: &mut &str) -> Vec<Json> {
    let mut results = Vec::new();

    while !input.is_empty() {
        let checkpoint = input.checkpoint();
        match json_value(input) {
            Ok(value) => results.push(value),
            Err(_) => {
                input.reset(&checkpoint);
                // Skip one character and try again
                if !input.is_empty() {
                    *input = &input[1..];
                }
            }
        }
    }

    results
}
```

winnow's checkpoint system enables error recovery and backtracking when needed. Creating a checkpoint saves the current input position, allowing the parser to reset on failure. This mechanism supports fault-tolerant parsing where partial results remain valuable, such as in IDE syntax highlighting or data extraction from corrupted inputs.

The checkpoint approach provides explicit control over backtracking, making performance implications clear. Unlike implicit backtracking, checkpoints document where recovery might occur, helping maintain predictable parser performance.

## Custom Input Types

```rust
use winnow::stream::Stream;
use winnow::token::any;

fn parse_tokens<'a>(tokens: &mut &'a [Token]) -> PResult<Ast> {
    let token = any.verify(|t: &Token| t.kind == TokenKind::Identifier);
    // Continue building parser with token-based input
}
```

winnow supports custom input types through the Stream trait, enabling parsing of pre-tokenized input or binary formats. Token-based parsing separates lexical analysis from syntax analysis, often improving performance and error messages. The any combinator consumes single elements from any stream type, while verify adds conditions without consuming extra input.

Custom streams enable specialized parsing scenarios like network protocols with length-prefixed fields or ast transformations where the input is already structured. The consistent combinator interface works across all stream types, allowing parser logic to remain unchanged when switching input representations.

## Performance Considerations

```rust
use winnow::combinator::cut_err;

fn json_array_cut(input: &mut &str) -> PResult<Json> {
    delimited(
        '[',
        cut_err(delimited(
            space0,
            separated(0.., json_value, delimited(space0, ',', space0)),
            space0,
        )),
        ']',
    )
    .map(Json::Array)
    .parse_next(input)
}
```

The cut_err combinator improves performance by preventing backtracking after a certain point. Once the opening bracket is matched, cut_err commits to parsing an array, eliminating unnecessary backtracking attempts. This optimization significantly improves performance for deeply nested structures while providing clearer error messages.

winnow's mutable reference approach eliminates allocation overhead associated with returning remaining input. Parsers operate directly on string slices without copying, maintaining zero-copy parsing throughout. The fold combinator builds results incrementally without intermediate collections, reducing memory allocation in repetitive parsers.

## Testing Strategies

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_evaluation() {
        let cases = vec![
            ("42", 42.0),
            ("1 + 2", 3.0),
            ("1 + 2 * 3", 7.0),
            ("(1 + 2) * 3", 9.0),
        ];

        for (input, expected) in cases {
            let expr = parse_expression(input).unwrap();
            assert_eq!(expr.eval(), expected);
        }
    }

    #[test]
    fn test_partial_parse() {
        let mut input = "123 extra";
        let num = number(&mut input).unwrap();
        assert_eq!(num, 123.0);
        assert_eq!(input, " extra");
    }

    #[test]
    fn test_error_recovery() {
        let mut input = "[1, invalid, 3]";
        let result = parse_with_recovery(&mut input);
        assert!(!result.is_empty());
    }
}
```

Testing winnow parsers involves validating both complete and partial parsing scenarios. The mutable reference approach makes it easy to test partial parsing, verifying that parsers consume exactly the expected input. Tests can examine the remaining input after parsing, ensuring parsers stop at appropriate boundaries.

Error recovery testing validates that parsers handle malformed input gracefully. Recovery tests ensure parsers extract valid portions from partially correct input, important for tooling that must handle incomplete or incorrect code. The checkpoint system makes it straightforward to test recovery strategies.

## Best Practices

Design parsers with clear separation between lexical and syntactic concerns. Use whitespace handling combinators consistently throughout the grammar rather than embedding whitespace in every parser. This separation simplifies both the grammar and error messages while making the parser's intent clearer.

Leverage winnow's mutable reference model for cleaner parser composition. The automatic position tracking eliminates manual state threading while making parser behavior more predictable. Use the checkpoint system sparingly, only where error recovery provides clear value.

Apply context annotations throughout the parser to improve error messages. Good context labels describe what the parser expects in user terms rather than implementation details. Layer contexts from general to specific, providing users with navigable error information.

Choose appropriate combinators for each parsing task. Use cut_err to commit to parse paths once initial markers are recognized. Apply fold for building accumulative results without intermediate collections. Select separated for delimiter-separated lists rather than manually handling separators.

winnow provides an elegant and performant approach to parsing that balances simplicity with power. Its mutable reference model and thoughtful combinator design create parsers that are both efficient and maintainable. The library's focus on ergonomics and error reporting makes it an excellent choice for building robust parsers for compilers, data formats, and domain-specific languages.