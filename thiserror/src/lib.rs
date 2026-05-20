//! # Compiler Error Handling with thiserror-context
//!
//! `thiserror-context` wraps a `thiserror` enum so that free-form context can
//! be layered on with `.context(...)` and `.with_context(|| ...)` (the
//! `anyhow`-style API) without erasing the underlying error type. The result
//! prints like `anyhow` but can still be pattern-matched like `thiserror`.
//!
//! The example below models a tiny two-phase compiler: a lexer and a parser
//! that consumes its output. Each phase has its own context-aware error type;
//! a third, top-level `CompilerError` wraps either of them while preserving
//! every context layer added along the way.

use std::ops::Range;

use thiserror::Error;
use thiserror_context::{impl_context, impl_from_carry_context, Context};

/// Byte-range source span attached to errors that know where they happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

/// Plain `thiserror` enum for the lexer. The "inner" half of the pair.
#[derive(Debug, Error)]
pub enum LexerErrorInner {
    #[error("unexpected character '{ch}' at byte {pos}")]
    UnexpectedChar { ch: char, pos: usize },

    #[error("invalid number literal '{literal}'")]
    InvalidNumber { literal: String },

    #[error("unterminated string literal at {span:?}")]
    UnterminatedString { span: Span },
}

// Generates `pub enum LexerError { Base(LexerErrorInner), Context { .. } }`
// plus `From<T: Into<LexerErrorInner>>`, Display, Debug, AsRef, and the
// blanket Context impl that powers `.context(...)` on `Result<_, LexerError>`.
impl_context!(LexerError(LexerErrorInner));

/// Plain `thiserror` enum for the parser.
#[derive(Debug, Error)]
pub enum ParserErrorInner {
    #[error("expected {expected}, found {found}")]
    UnexpectedToken { expected: String, found: String },

    #[error("missing {item}")]
    MissingItem { item: String },

    #[error("upstream lexer error")]
    Lexer(LexerError),
}

impl_context!(ParserError(ParserErrorInner));

// Bridge LexerError into ParserError while *preserving* any context layers
// already attached to the lexer error. Note: no `#[from]` on the `Lexer`
// variant above; this macro owns the conversion.
impl_from_carry_context!(LexerError, ParserError, ParserErrorInner::Lexer);

/// Top-level error covering every phase.
#[derive(Debug, Error)]
pub enum CompilerErrorInner {
    #[error("lexer phase failed")]
    Lexer(LexerError),

    #[error("parser phase failed")]
    Parser(ParserError),
}

impl_context!(CompilerError(CompilerErrorInner));
impl_from_carry_context!(LexerError, CompilerError, CompilerErrorInner::Lexer);
impl_from_carry_context!(ParserError, CompilerError, CompilerErrorInner::Parser);

/// Read the first identifier from `input`, returning a lexer error if the
/// first character is not alphabetic. Demonstrates `.context()` at the
/// innermost layer.
pub fn lex_identifier(input: &str, pos: usize) -> Result<String, LexerError> {
    let first = input
        .chars()
        .next()
        .ok_or(LexerErrorInner::UnexpectedChar { ch: '\0', pos })
        .context("expected identifier")?;

    if !first.is_alphabetic() && first != '_' {
        return Err(LexerErrorInner::UnexpectedChar { ch: first, pos })
            .context("identifier must start with a letter or underscore")?;
    }

    Ok(input
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect())
}

/// Parse `let <name> = <expr>` from a token slice. Calls `lex_identifier`
/// internally; any lexer-side context propagates through the auto-generated
/// `From<LexerError> for ParserError` impl.
pub fn parse_let_binding(tokens: &[&str]) -> Result<(String, String), ParserError> {
    let [keyword, name, eq, value, ..] = tokens else {
        return Err(ParserErrorInner::MissingItem {
            item: "complete let binding".into(),
        })
        .context("parsing let binding")?;
    };

    if *keyword != "let" {
        return Err(ParserErrorInner::UnexpectedToken {
            expected: "'let'".into(),
            found: (*keyword).into(),
        })
        .context("at start of binding")?;
    }

    // `?` auto-converts `LexerError` into `ParserError` through the
    // `impl_from_carry_context!` bridge, preserving every layer of context the
    // lexer attached. (We avoid `.with_context()` here because it would be
    // ambiguous between two wrapper targets.)
    let bound = lex_identifier(name, 0)?;

    if *eq != "=" {
        return Err(ParserErrorInner::UnexpectedToken {
            expected: "'='".into(),
            found: (*eq).into(),
        })
        .context("after binding name")?;
    }

    Ok((bound, (*value).into()))
}

/// Drive the whole pipeline. Lex first, then parse; either failure becomes a
/// `CompilerError` with every context layer intact.
pub fn compile(input: &str) -> Result<(String, String), CompilerError> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    parse_let_binding(&tokens)
        .map_err(CompilerError::from)
        .context("compiling input")
        .with_context(|| format!("source: {input:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_layers_show_up_in_debug() {
        let err = parse_let_binding(&["if"]).unwrap_err();
        let debug = format!("{err:?}");
        assert!(debug.contains("MissingItem"));
        assert!(debug.contains("parsing let binding"));
    }

    #[test]
    fn as_ref_pattern_matches_through_context() {
        let err = parse_let_binding(&["fn", "x", "=", "1"]).unwrap_err();
        match err.as_ref() {
            ParserErrorInner::UnexpectedToken { expected, .. } => assert_eq!(expected, "'let'"),
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn cross_phase_conversion_preserves_context() {
        // Force a lexer error inside the parser path. The context attached by
        // `lex_identifier` survives the `LexerError -> ParserError` conversion
        // because `impl_from_carry_context!` walks the context chain.
        let err = parse_let_binding(&["let", "1bad", "=", "x"]).unwrap_err();
        let debug = format!("{err:?}");
        assert!(debug.contains("UnexpectedChar"));
        assert!(debug.contains("identifier must start"));
    }

    #[test]
    fn full_pipeline_wraps_into_compiler_error() {
        let err = compile("let 1bad = x").unwrap_err();
        let debug = format!("{err:?}");
        assert!(debug.contains("compiling input"));
        assert!(debug.contains("source:"));
    }
}
