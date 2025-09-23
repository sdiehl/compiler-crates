use std::ops::Range;

use rustc_lexer::{self, Base, LiteralKind, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: Range<usize>,
}

pub struct Lexer<'input> {
    input: &'input str,
    position: usize,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self { input, position: 0 }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            let remaining = &self.input[self.position..];
            let token = rustc_lexer::first_token(remaining);

            let start = self.position;
            let end = self.position + token.len as usize;
            let text = self.input[start..end].to_string();

            // Skip whitespace and comments unless we're preserving them
            match token.kind {
                TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment { .. } => {
                    self.position = end;
                    continue;
                }
                _ => {}
            }

            tokens.push(Token {
                kind: token.kind,
                text,
                span: start..end,
            });

            self.position = end;
        }

        tokens
    }

    pub fn tokenize_with_trivia(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            let remaining = &self.input[self.position..];
            let token = rustc_lexer::first_token(remaining);

            let start = self.position;
            let end = self.position + token.len as usize;
            let text = self.input[start..end].to_string();

            tokens.push(Token {
                kind: token.kind,
                text,
                span: start..end,
            });

            self.position = end;
        }

        tokens
    }
}

pub fn strip_shebang(input: &str) -> &str {
    rustc_lexer::strip_shebang(input)
        .map(|shebang_len| &input[shebang_len..])
        .unwrap_or(input)
}

pub fn cook_lexer_literal(
    kind: LiteralKind,
    text: &str,
    _start: usize,
) -> Result<ParsedLiteral, LiteralError> {
    match kind {
        LiteralKind::Int { base, empty_int } => {
            if empty_int {
                return Err(LiteralError::EmptyInt);
            }

            let text = text.replace('_', "");
            let value = match base {
                Base::Binary => u128::from_str_radix(&text[2..], 2),
                Base::Octal => u128::from_str_radix(&text[2..], 8),
                Base::Decimal => text.parse(),
                Base::Hexadecimal => u128::from_str_radix(&text[2..], 16),
            };

            match value {
                Ok(n) => Ok(ParsedLiteral::Int(n)),
                Err(_) => Err(LiteralError::IntegerOverflow),
            }
        }

        LiteralKind::Float {
            base,
            empty_exponent,
        } => {
            if empty_exponent {
                return Err(LiteralError::EmptyExponent);
            }

            if base != Base::Decimal {
                return Err(LiteralError::NonDecimalFloat);
            }

            let text = text.replace('_', "");
            match text.parse() {
                Ok(f) => Ok(ParsedLiteral::Float(f)),
                Err(_) => Err(LiteralError::InvalidFloat),
            }
        }

        LiteralKind::Char { terminated } => {
            if !terminated {
                return Err(LiteralError::UnterminatedChar);
            }

            let content = &text[1..text.len() - 1];
            let unescaped = unescape_char(content)?;
            Ok(ParsedLiteral::Char(unescaped))
        }

        LiteralKind::Byte { terminated } => {
            if !terminated {
                return Err(LiteralError::UnterminatedByte);
            }

            let content = &text[2..text.len() - 1];
            let unescaped = unescape_byte(content)?;
            Ok(ParsedLiteral::Byte(unescaped))
        }

        LiteralKind::Str { terminated } => {
            if !terminated {
                return Err(LiteralError::UnterminatedString);
            }

            let content = &text[1..text.len() - 1];
            let unescaped = unescape_string(content)?;
            Ok(ParsedLiteral::Str(unescaped))
        }

        LiteralKind::ByteStr { terminated } => {
            if !terminated {
                return Err(LiteralError::UnterminatedByteString);
            }

            let content = &text[2..text.len() - 1];
            let unescaped = unescape_byte_string(content)?;
            Ok(ParsedLiteral::ByteStr(unescaped))
        }

        LiteralKind::RawStr {
            n_hashes,
            started,
            terminated,
        } => {
            if !started || !terminated {
                return Err(LiteralError::UnterminatedRawString);
            }

            let _hashes = "#".repeat(n_hashes);
            let start = 2 + n_hashes;
            let end = text.len() - n_hashes - 1;
            let content = text[start..end].to_string();
            Ok(ParsedLiteral::RawStr(content))
        }

        LiteralKind::RawByteStr {
            n_hashes,
            started,
            terminated,
        } => {
            if !started || !terminated {
                return Err(LiteralError::UnterminatedRawByteString);
            }

            let _hashes = "#".repeat(n_hashes);
            let start = 3 + n_hashes;
            let end = text.len() - n_hashes - 1;
            let content = text.as_bytes()[start..end].to_vec();
            Ok(ParsedLiteral::RawByteStr(content))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedLiteral {
    Int(u128),
    Float(f64),
    Char(char),
    Byte(u8),
    Str(String),
    ByteStr(Vec<u8>),
    RawStr(String),
    RawByteStr(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralError {
    EmptyInt,
    IntegerOverflow,
    EmptyExponent,
    NonDecimalFloat,
    InvalidFloat,
    UnterminatedChar,
    UnterminatedByte,
    UnterminatedString,
    UnterminatedByteString,
    UnterminatedRawString,
    UnterminatedRawByteString,
    InvalidEscape(String),
}

// Simplified escape handling - in real compiler this would be much more
// comprehensive
fn unescape_char(s: &str) -> Result<char, LiteralError> {
    if let Some(stripped) = s.strip_prefix('\\') {
        match stripped {
            "n" => Ok('\n'),
            "r" => Ok('\r'),
            "t" => Ok('\t'),
            "\\" => Ok('\\'),
            "'" => Ok('\''),
            "\"" => Ok('"'),
            "0" => Ok('\0'),
            _ => Err(LiteralError::InvalidEscape(s.to_string())),
        }
    } else if s.len() == 1 {
        Ok(s.chars().next().unwrap())
    } else {
        Err(LiteralError::InvalidEscape(s.to_string()))
    }
}

fn unescape_byte(s: &str) -> Result<u8, LiteralError> {
    unescape_char(s).and_then(|c| {
        if c as u32 <= 255 {
            Ok(c as u8)
        } else {
            Err(LiteralError::InvalidEscape(s.to_string()))
        }
    })
}

fn unescape_string(s: &str) -> Result<String, LiteralError> {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    '\\' => result.push('\\'),
                    '\'' => result.push('\''),
                    '"' => result.push('"'),
                    '0' => result.push('\0'),
                    _ => return Err(LiteralError::InvalidEscape(format!("\\{}", next))),
                }
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

fn unescape_byte_string(s: &str) -> Result<Vec<u8>, LiteralError> {
    unescape_string(s).map(|s| s.into_bytes())
}

pub fn tokenize_and_validate(input: &str) -> Result<Vec<Token>, Vec<ValidationError>> {
    let mut lexer = Lexer::new(input);
    let mut errors = Vec::new();
    let tokens = lexer.tokenize_with_trivia();

    for (i, token) in tokens.iter().enumerate() {
        match &token.kind {
            TokenKind::Unknown => {
                errors.push(ValidationError {
                    token_index: i,
                    kind: ValidationErrorKind::UnknownToken,
                    span: token.span.clone(),
                });
            }
            TokenKind::Literal { kind, .. } => {
                if let Err(e) = cook_lexer_literal(*kind, &token.text, token.span.start) {
                    errors.push(ValidationError {
                        token_index: i,
                        kind: ValidationErrorKind::InvalidLiteral(e),
                        span: token.span.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub token_index: usize,
    pub kind: ValidationErrorKind,
    pub span: Range<usize>,
}

#[derive(Debug, Clone)]
pub enum ValidationErrorKind {
    UnknownToken,
    InvalidLiteral(LiteralError),
}

pub fn is_whitespace(kind: TokenKind) -> bool {
    matches!(kind, TokenKind::Whitespace)
}

pub fn is_comment(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::LineComment | TokenKind::BlockComment { .. }
    )
}

pub fn is_literal(kind: TokenKind) -> bool {
    matches!(kind, TokenKind::Literal { .. })
}

pub fn describe_token(kind: TokenKind) -> &'static str {
    match kind {
        TokenKind::Ident => "identifier",
        TokenKind::RawIdent => "raw identifier",
        TokenKind::Literal { kind, .. } => match kind {
            LiteralKind::Int { .. } => "integer literal",
            LiteralKind::Float { .. } => "float literal",
            LiteralKind::Char { .. } => "character literal",
            LiteralKind::Byte { .. } => "byte literal",
            LiteralKind::Str { .. } => "string literal",
            LiteralKind::ByteStr { .. } => "byte string literal",
            LiteralKind::RawStr { .. } => "raw string literal",
            LiteralKind::RawByteStr { .. } => "raw byte string literal",
        },
        TokenKind::Lifetime { .. } => "lifetime",
        TokenKind::Semi => "semicolon",
        TokenKind::Comma => "comma",
        TokenKind::Dot => "dot",
        TokenKind::OpenParen => "open parenthesis",
        TokenKind::CloseParen => "close parenthesis",
        TokenKind::OpenBrace => "open brace",
        TokenKind::CloseBrace => "close brace",
        TokenKind::OpenBracket => "open bracket",
        TokenKind::CloseBracket => "close bracket",
        TokenKind::At => "at sign",
        TokenKind::Pound => "pound sign",
        TokenKind::Tilde => "tilde",
        TokenKind::Question => "question mark",
        TokenKind::Colon => "colon",
        TokenKind::Dollar => "dollar sign",
        TokenKind::Eq => "equals",
        TokenKind::Lt => "less than",
        TokenKind::Gt => "greater than",
        TokenKind::Minus => "minus",
        TokenKind::And => "ampersand",
        TokenKind::Or => "pipe",
        TokenKind::Plus => "plus",
        TokenKind::Star => "star",
        TokenKind::Slash => "slash",
        TokenKind::Caret => "caret",
        TokenKind::Percent => "percent",
        TokenKind::Unknown => "unknown token",
        TokenKind::Not => "exclamation mark",
        TokenKind::Whitespace => "whitespace",
        TokenKind::LineComment => "line comment",
        TokenKind::BlockComment { .. } => "block comment",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let input = "fn main() { let x = 42; }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].text, "fn");
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].text, "main");
        assert_eq!(tokens[2].kind, TokenKind::OpenParen);
        assert_eq!(tokens[3].kind, TokenKind::CloseParen);
        assert_eq!(tokens[4].kind, TokenKind::OpenBrace);
    }

    #[test]
    fn test_literals() {
        let input = r##"42 3.14 'a' b'x' "hello" b"bytes" r#"raw"#"##;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        // Check that all are literals
        for token in &tokens {
            assert!(is_literal(token.kind));
        }
    }

    #[test]
    fn test_trivia_handling() {
        let input = "// comment\nfn /* block */ main()";
        let mut lexer = Lexer::new(input);

        // Without trivia
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 4); // fn main ( )

        // With trivia
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_with_trivia();
        assert!(tokens.len() > 4); // includes comments and whitespace
    }

    #[test]
    fn test_shebang() {
        let input = "#!/usr/bin/env rust\nfn main() {}";
        let stripped = strip_shebang(input);
        // The newline is included after stripping the shebang
        assert!(stripped.starts_with("\nfn main()"));
    }

    #[test]
    fn test_literal_parsing() {
        let cases = vec![
            (
                LiteralKind::Int {
                    base: Base::Decimal,
                    empty_int: false,
                },
                "42",
                ParsedLiteral::Int(42),
            ),
            (
                LiteralKind::Int {
                    base: Base::Hexadecimal,
                    empty_int: false,
                },
                "0xFF",
                ParsedLiteral::Int(255),
            ),
            (
                LiteralKind::Float {
                    base: Base::Decimal,
                    empty_exponent: false,
                },
                "3.14",
                ParsedLiteral::Float(3.14),
            ),
            (
                LiteralKind::Char { terminated: true },
                "'a'",
                ParsedLiteral::Char('a'),
            ),
        ];

        for (kind, text, expected) in cases {
            let result = cook_lexer_literal(kind, text, 0).unwrap();
            assert_eq!(result, expected);
        }
    }
}
