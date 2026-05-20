//! Example: feeding a logos lexer through `marginalia::TriviaLexer` so that
//! comments and blank lines are recorded on the side while the parser sees a
//! clean stream of semantic tokens.

use logos::Logos;
use marginalia::{Classify, Trivia, TriviaEvent, TriviaLexer, TriviaPiece};

/// A small calculator token enum. `LineComment` and `BlockComment` carry their
/// payload so the `Classify` impl can hand the text to marginalia.
#[derive(Clone, Debug, Logos, PartialEq, Eq)]
#[logos(skip r"[ \t\f\r\n]+")]
pub enum Tok {
    #[token("let")]
    Let,
    #[token("=")]
    Eq,
    #[token(";")]
    Semi,
    #[token("+")]
    Plus,
    #[token("*")]
    Star,

    #[regex(r"[0-9]+", |l| l.slice().parse::<i64>().ok())]
    Num(i64),

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |l| l.slice().to_owned(), priority = 2)]
    Ident(String),

    #[regex(r"//[^\n]*", |l| l.slice().to_owned())]
    LineComment(String),

    #[regex(r"/\*([^*]|\*[^/])*\*/", |l| l.slice().to_owned())]
    BlockComment(String),
}

/// Hand each comment variant to marginalia. Non-trivia tokens return `None` and
/// flow through the lexer unchanged.
impl Classify for Tok {
    fn trivia(&self) -> Option<TriviaPiece<'_>> {
        match self {
            Self::LineComment(s) => Some(TriviaPiece {
                kind: marginalia::TriviaKind::Line,
                text: s,
            }),
            Self::BlockComment(s) => Some(TriviaPiece {
                kind: marginalia::TriviaKind::Block,
                text: s,
            }),
            _ => None,
        }
    }
}

/// Lex `source`, returning the semantic tokens (comments stripped) and the
/// trivia table that the formatter or AST-attachment pass would consume next.
pub fn lex(source: &str) -> (Vec<(usize, Tok, usize)>, Vec<TriviaEvent>) {
    let raw = Tok::lexer(source).spanned().map(|(res, span)| match res {
        Ok(tok) => Ok((span.start, tok, span.end)),
        Err(()) => Err(()),
    });
    let mut layer = TriviaLexer::new(raw, source);
    let tokens: Vec<_> = (&mut layer).filter_map(Result::ok).collect();
    let table = layer.into_table();
    (tokens, table.events().to_vec())
}

/// Project a trivia event to a short human-readable tag for snapshot tests.
pub fn describe(event: &TriviaEvent) -> String {
    match &event.trivia {
        Trivia::Line(text) => format!("line@{}..{}: {text}", event.span.start, event.span.end),
        Trivia::Block(text) => format!("block@{}..{}: {text}", event.span.start, event.span.end),
        Trivia::BlankLine => format!("blank@{}..{}", event.span.start, event.span.end),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_never_sees_comments() {
        let src = "let x = 1; // tail\n\n/* between */ x + 2;";
        let (tokens, _) = lex(src);
        for (_, tok, _) in &tokens {
            assert!(!matches!(tok, Tok::LineComment(_) | Tok::BlockComment(_)));
        }
        assert!(tokens.iter().any(|(_, t, _)| matches!(t, Tok::Let)));
    }

    #[test]
    fn trivia_table_captures_comments_and_blank_lines() {
        let src = "let x = 1; // tail\n\n/* between */ x + 2;";
        let (_, events) = lex(src);
        let kinds: Vec<_> = events.iter().map(describe).collect();
        assert!(kinds.iter().any(|s| s.starts_with("line@")));
        assert!(kinds.iter().any(|s| s.starts_with("blank@")));
        assert!(kinds.iter().any(|s| s.starts_with("block@")));
    }
}
