use std::ops::Range;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{self, Config};
use termcolor::{ColorChoice, StandardStream};

/// A compiler diagnostic system built on codespan-reporting
pub struct DiagnosticEngine {
    files: SimpleFiles<String, String>,
    config: Config,
}

impl DiagnosticEngine {
    pub fn new() -> Self {
        Self {
            files: SimpleFiles::new(),
            config: Config::default(),
        }
    }

    pub fn add_file(&mut self, name: String, source: String) -> usize {
        self.files.add(name, source)
    }

    pub fn emit_diagnostic(&self, diagnostic: Diagnostic<usize>) {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let _ = term::emit(&mut writer.lock(), &self.config, &self.files, &diagnostic);
    }
}

impl Default for DiagnosticEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Type system for a simple functional language
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    String,
    Function(Box<Type>, Box<Type>),
    List(Box<Type>),
    Unknown,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Function(from, to) => write!(f, "{} -> {}", from, to),
            Type::List(elem) => write!(f, "[{}]", elem),
            Type::Unknown => write!(f, "_"),
        }
    }
}

/// Common compiler errors with location information
#[derive(Debug, Clone)]
pub enum CompilerError {
    TypeMismatch {
        expected: Type,
        found: Type,
        location: Range<usize>,
    },
    UndefinedVariable {
        name: String,
        location: Range<usize>,
        similar: Vec<String>,
    },
    ParseError {
        message: String,
        location: Range<usize>,
        hint: Option<String>,
    },
    DuplicateDefinition {
        name: String,
        first_location: Range<usize>,
        second_location: Range<usize>,
    },
}

impl CompilerError {
    pub fn to_diagnostic(&self, file_id: usize) -> Diagnostic<usize> {
        match self {
            CompilerError::TypeMismatch {
                expected,
                found,
                location,
            } => Diagnostic::error()
                .with_message("type mismatch")
                .with_labels(vec![Label::primary(file_id, location.clone())
                    .with_message(format!("expected `{}`, found `{}`", expected, found))]),

            CompilerError::UndefinedVariable {
                name,
                location,
                similar,
            } => {
                let mut diagnostic = Diagnostic::error()
                    .with_message(format!("undefined variable `{}`", name))
                    .with_labels(vec![Label::primary(file_id, location.clone())
                        .with_message("not found in scope")]);

                if !similar.is_empty() {
                    let suggestions = similar.join(", ");
                    diagnostic =
                        diagnostic.with_notes(vec![format!("did you mean: {}?", suggestions)]);
                }

                diagnostic
            }

            CompilerError::ParseError {
                message,
                location,
                hint,
            } => {
                let mut diagnostic =
                    Diagnostic::error()
                        .with_message("parse error")
                        .with_labels(vec![
                            Label::primary(file_id, location.clone()).with_message(message)
                        ]);

                if let Some(hint) = hint {
                    diagnostic = diagnostic.with_notes(vec![hint.clone()]);
                }

                diagnostic
            }

            CompilerError::DuplicateDefinition {
                name,
                first_location,
                second_location,
            } => Diagnostic::error()
                .with_message(format!("duplicate definition of `{}`", name))
                .with_labels(vec![
                    Label::secondary(file_id, first_location.clone())
                        .with_message("first definition here"),
                    Label::primary(file_id, second_location.clone()).with_message("redefined here"),
                ]),
        }
    }
}

/// A simple lexer for demonstration purposes
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number(i64),
    Identifier(String),
    Let,
    If,
    Else,
    Function,
    Arrow,
    LeftParen,
    RightParen,
    Equals,
    Plus,
    Minus,
    Star,
    Slash,
    Less,
    Greater,
    Bang,
    Semicolon,
    Eof,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, CompilerError> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            self.skip_whitespace();
            if self.position >= self.input.len() {
                break;
            }

            let start = self.position;
            let token = self.next_token()?;
            let end = self.position;

            tokens.push(Token {
                kind: token,
                span: start..end,
            });
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            span: self.position..self.position,
        });

        Ok(tokens)
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len()
            && self.input.as_bytes()[self.position].is_ascii_whitespace()
        {
            self.position += 1;
        }
    }

    fn next_token(&mut self) -> Result<TokenKind, CompilerError> {
        let start = self.position;
        let ch = self.current_char();

        match ch {
            '0'..='9' => self.read_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(),
            '+' => {
                self.advance();
                Ok(TokenKind::Plus)
            }
            '-' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    Ok(TokenKind::Arrow)
                } else {
                    Ok(TokenKind::Minus)
                }
            }
            '*' => {
                self.advance();
                Ok(TokenKind::Star)
            }
            '/' => {
                self.advance();
                Ok(TokenKind::Slash)
            }
            '<' => {
                self.advance();
                Ok(TokenKind::Less)
            }
            '>' => {
                self.advance();
                Ok(TokenKind::Greater)
            }
            '!' => {
                self.advance();
                Ok(TokenKind::Bang)
            }
            '=' => {
                self.advance();
                Ok(TokenKind::Equals)
            }
            '(' => {
                self.advance();
                Ok(TokenKind::LeftParen)
            }
            ')' => {
                self.advance();
                Ok(TokenKind::RightParen)
            }
            ';' => {
                self.advance();
                Ok(TokenKind::Semicolon)
            }
            _ => Err(CompilerError::ParseError {
                message: format!("unexpected character `{}`", ch),
                location: start..start + 1,
                hint: Some("expected a number, identifier, or operator".to_string()),
            }),
        }
    }

    fn current_char(&self) -> char {
        self.input.as_bytes()[self.position] as char
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn read_number(&mut self) -> Result<TokenKind, CompilerError> {
        let start = self.position;
        while self.position < self.input.len()
            && self.input.as_bytes()[self.position].is_ascii_digit()
        {
            self.position += 1;
        }
        let num_str = &self.input[start..self.position];
        let num = num_str.parse().map_err(|_| CompilerError::ParseError {
            message: "Invalid number format".to_string(),
            location: start..self.position,
            hint: Some("Number too large to parse".to_string()),
        })?;
        Ok(TokenKind::Number(num))
    }

    fn read_identifier(&mut self) -> Result<TokenKind, CompilerError> {
        let start = self.position;
        while self.position < self.input.len() {
            let ch = self.input.as_bytes()[self.position];
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                self.position += 1;
            } else {
                break;
            }
        }
        let ident = &self.input[start..self.position];
        let kind = match ident {
            "let" => TokenKind::Let,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "fn" => TokenKind::Function,
            _ => TokenKind::Identifier(ident.to_string()),
        };
        Ok(kind)
    }
}

/// Multi-file project support
pub struct Project {
    engine: DiagnosticEngine,
    file_ids: Vec<(String, usize)>,
}

impl Project {
    pub fn new() -> Self {
        Self {
            engine: DiagnosticEngine::new(),
            file_ids: Vec::new(),
        }
    }

    pub fn add_file(&mut self, path: String, content: String) -> usize {
        let file_id = self.engine.add_file(path.clone(), content);
        self.file_ids.push((path, file_id));
        file_id
    }

    pub fn compile(&self) -> Result<(), Vec<Diagnostic<usize>>> {
        let mut diagnostics = Vec::new();

        // Simulate compilation with various error types
        for (path, file_id) in &self.file_ids {
            if path.ends_with("types.ml") {
                // Type error example
                diagnostics.push(
                    CompilerError::TypeMismatch {
                        expected: Type::Int,
                        found: Type::String,
                        location: 45..52,
                    }
                    .to_diagnostic(*file_id),
                );
            } else if path.ends_with("undefined.ml") {
                // Undefined variable with suggestions
                diagnostics.push(
                    CompilerError::UndefinedVariable {
                        name: "lenght".to_string(),
                        location: 23..29,
                        similar: vec!["length".to_string(), "len".to_string()],
                    }
                    .to_diagnostic(*file_id),
                );
            }
        }

        if diagnostics.is_empty() {
            Ok(())
        } else {
            for diagnostic in &diagnostics {
                self.engine.emit_diagnostic(diagnostic.clone());
            }
            Err(diagnostics)
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a warning diagnostic
pub fn create_warning(
    file_id: usize,
    message: &str,
    location: Range<usize>,
    note: Option<String>,
) -> Diagnostic<usize> {
    let mut diagnostic = Diagnostic::warning()
        .with_message(message)
        .with_labels(vec![Label::primary(file_id, location)]);

    if let Some(note) = note {
        diagnostic = diagnostic.with_notes(vec![note]);
    }

    diagnostic
}

/// Create an information diagnostic
pub fn create_info(file_id: usize, message: &str, location: Range<usize>) -> Diagnostic<usize> {
    Diagnostic::note()
        .with_message(message)
        .with_labels(vec![Label::primary(file_id, location)])
}

#[cfg(test)]
mod tests {
    use codespan_reporting::diagnostic::Severity;

    use super::*;

    #[test]
    fn test_lexer() {
        let mut lexer = Lexer::new("let x = 42 + 3");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 7); // let x = 42 + 3 EOF
    }

    #[test]
    fn test_type_display() {
        let int_to_bool = Type::Function(Box::new(Type::Int), Box::new(Type::Bool));
        assert_eq!(int_to_bool.to_string(), "int -> bool");

        let list_of_ints = Type::List(Box::new(Type::Int));
        assert_eq!(list_of_ints.to_string(), "[int]");
    }

    #[test]
    fn test_diagnostic_creation() {
        let error = CompilerError::TypeMismatch {
            expected: Type::Int,
            found: Type::Bool,
            location: 10..15,
        };
        let diagnostic = error.to_diagnostic(0);
        assert_eq!(diagnostic.severity, Severity::Error);
    }
}
