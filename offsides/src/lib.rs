//! Example: feeding a logos lexer through `offsides::LayoutLexer` so the
//! downstream grammar sees explicit virtual `v{`, `v;`, `v}` tokens instead of
//! having to reason about indentation columns.

use logos::Logos;
use offsides::{Layout, LayoutConfig, LayoutLexer, LayoutMode};

/// Token enum for a tiny block-let calculator. `VOpen`/`VSemi`/`VClose` are
/// the virtual variants that `LayoutLexer` splices into the stream; the parser
/// matches on them like real braces and semicolons.
#[derive(Clone, Debug, Logos, PartialEq, Eq)]
#[logos(skip r"[ \t\f\r\n]+")]
pub enum Tok {
    #[token("let")]
    Let,
    #[token("in")]
    In,
    #[token("=")]
    Eq,
    #[token("+")]
    Plus,
    #[token("*")]
    Star,

    #[regex(r"[0-9]+", |l| l.slice().parse::<i64>().ok())]
    Num(i64),

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |l| l.slice().to_owned(), priority = 2)]
    Ident(String),

    VOpen,
    VClose,
    VSemi,
}

/// Map the three required virtual constructors onto our enum variants.
impl Layout for Tok {
    fn v_open() -> Self {
        Self::VOpen
    }
    fn v_close() -> Self {
        Self::VClose
    }
    fn v_sep() -> Self {
        Self::VSemi
    }
}

/// `let` is the sole opener: the next token's column sets the indent level.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_opener(t: &Tok) -> bool {
    matches!(t, Tok::Let)
}

/// Lex `source` and return the post-layout token stream. The downstream grammar
/// can treat `VOpen`/`VSemi`/`VClose` as ordinary punctuation.
pub fn lex(source: &str) -> Vec<Tok> {
    let raw = Tok::lexer(source).spanned().map(|(res, span)| match res {
        Ok(tok) => Ok((span.start, tok, span.end)),
        Err(()) => Err(()),
    });
    let cfg = LayoutConfig::new(is_opener);
    LayoutLexer::new(raw, source, cfg)
        .filter_map(Result::ok)
        .map(|(_, t, _)| t)
        .collect()
}

/// Same input under `Eager` mode: top-level itself is one layout block.
pub fn lex_eager(source: &str) -> Vec<Tok> {
    let raw = Tok::lexer(source).spanned().map(|(res, span)| match res {
        Ok(tok) => Ok((span.start, tok, span.end)),
        Err(()) => Err(()),
    });
    let cfg = LayoutConfig::new(is_opener).with_mode(LayoutMode::Eager);
    LayoutLexer::new(raw, source, cfg)
        .filter_map(Result::ok)
        .map(|(_, t, _)| t)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lazy_mode_inserts_virtual_braces_around_let_block() {
        let src = "let\n  x = 1\n  y = 2\nin x + y";
        let out = lex(src);
        assert_eq!(
            out,
            vec![
                Tok::Let,
                Tok::VOpen,
                Tok::Ident("x".into()),
                Tok::Eq,
                Tok::Num(1),
                Tok::VSemi,
                Tok::Ident("y".into()),
                Tok::Eq,
                Tok::Num(2),
                Tok::VClose,
                Tok::In,
                Tok::Ident("x".into()),
                Tok::Plus,
                Tok::Ident("y".into()),
            ]
        );
    }

    #[test]
    fn eager_mode_wraps_top_level() {
        let src = "x\ny\nz";
        let out = lex_eager(src);
        assert_eq!(
            out,
            vec![
                Tok::VOpen,
                Tok::Ident("x".into()),
                Tok::VSemi,
                Tok::Ident("y".into()),
                Tok::VSemi,
                Tok::Ident("z".into()),
                Tok::VClose,
            ]
        );
    }
}
