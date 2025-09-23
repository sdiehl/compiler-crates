use std::collections::HashMap;
use std::fmt;

use codespan::{ByteIndex, ByteOffset, ColumnIndex, LineIndex, LineOffset, Span};

/// A source file with span tracking
#[derive(Debug, Clone)]
pub struct SourceFile {
    name: String,
    contents: String,
    line_starts: Vec<ByteIndex>,
}

impl SourceFile {
    pub fn new(name: String, contents: String) -> Self {
        let line_starts = std::iter::once(ByteIndex::from(0))
            .chain(contents.char_indices().filter_map(|(i, c)| {
                if c == '\n' {
                    Some(ByteIndex::from(i as u32 + 1))
                } else {
                    None
                }
            }))
            .collect();

        Self {
            name,
            contents,
            line_starts,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn contents(&self) -> &str {
        &self.contents
    }

    pub fn line_index(&self, byte_index: ByteIndex) -> LineIndex {
        match self.line_starts.binary_search(&byte_index) {
            Ok(line) => LineIndex::from(line as u32),
            Err(next_line) => LineIndex::from((next_line as u32).saturating_sub(1)),
        }
    }

    pub fn column_index(&self, byte_index: ByteIndex) -> ColumnIndex {
        let line_index = self.line_index(byte_index);
        let line_start = self.line_starts[line_index.to_usize()];
        let column_offset = byte_index - line_start;
        ColumnIndex::from(column_offset.to_usize() as u32)
    }

    pub fn location(&self, byte_index: ByteIndex) -> Location {
        Location {
            line: self.line_index(byte_index),
            column: self.column_index(byte_index),
        }
    }

    pub fn slice(&self, span: Span) -> &str {
        let start = span.start().to_usize();
        let end = span.end().to_usize();
        &self.contents[start..end]
    }
}

/// A location in a source file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: LineIndex,
    pub column: ColumnIndex,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.line.to_usize() + 1,
            self.column.to_usize() + 1
        )
    }
}

/// A span manager for tracking multiple source files
pub struct SpanManager {
    files: Vec<SourceFile>,
    file_map: HashMap<String, usize>,
}

impl SpanManager {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            file_map: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, name: String, contents: String) -> FileId {
        let file_id = FileId(self.files.len());
        let file = SourceFile::new(name.clone(), contents);
        self.files.push(file);
        self.file_map.insert(name, file_id.0);
        file_id
    }

    pub fn get_file(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(id.0)
    }

    pub fn find_file(&self, name: &str) -> Option<FileId> {
        self.file_map.get(name).map(|&id| FileId(id))
    }

    pub fn create_span(&self, start: ByteIndex, end: ByteIndex) -> Span {
        Span::new(start, end)
    }

    pub fn merge_spans(&self, first: Span, second: Span) -> Span {
        let start = first.start().min(second.start());
        let end = first.end().max(second.end());
        Span::new(start, end)
    }
}

impl Default for SpanManager {
    fn default() -> Self {
        Self::new()
    }
}

/// File identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(usize);

/// A token with span information
#[derive(Debug, Clone)]
pub struct Token<T> {
    pub kind: T,
    pub span: Span,
    pub file_id: FileId,
}

impl<T> Token<T> {
    pub fn new(kind: T, span: Span, file_id: FileId) -> Self {
        Self {
            kind,
            span,
            file_id,
        }
    }
}

/// Example token types for demonstration
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Number(i64),
    String(String),
    Keyword(Keyword),
    Operator(Operator),
    Delimiter(Delimiter),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Let,
    If,
    Else,
    While,
    Function,
    Return,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    NotEqual,
    Less,
    Greater,
    Assign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Delimiter {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,
}

/// A simple lexer using codespan for span tracking
pub struct Lexer {
    input: String,
    position: usize,
    file_id: FileId,
}

impl Lexer {
    pub fn new(input: String, file_id: FileId) -> Self {
        Self {
            input,
            position: 0,
            file_id,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token<TokenKind>> {
        let mut tokens = Vec::new();

        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() {
                break;
            }

            let start = ByteIndex::from(self.position as u32);

            if let Some(token) = self.scan_token() {
                let end = ByteIndex::from(self.position as u32);
                let span = Span::new(start, end);
                tokens.push(Token::new(token, span, self.file_id));
            }
        }

        tokens
    }

    fn scan_token(&mut self) -> Option<TokenKind> {
        let start_char = self.current_char()?;

        match start_char {
            '+' => {
                self.advance();
                Some(TokenKind::Operator(Operator::Plus))
            }
            '-' => {
                self.advance();
                Some(TokenKind::Operator(Operator::Minus))
            }
            '*' => {
                self.advance();
                Some(TokenKind::Operator(Operator::Star))
            }
            '/' => {
                self.advance();
                Some(TokenKind::Operator(Operator::Slash))
            }
            '=' => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    Some(TokenKind::Operator(Operator::Equal))
                } else {
                    Some(TokenKind::Operator(Operator::Assign))
                }
            }
            '!' => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    Some(TokenKind::Operator(Operator::NotEqual))
                } else {
                    None
                }
            }
            '<' => {
                self.advance();
                Some(TokenKind::Operator(Operator::Less))
            }
            '>' => {
                self.advance();
                Some(TokenKind::Operator(Operator::Greater))
            }
            '(' => {
                self.advance();
                Some(TokenKind::Delimiter(Delimiter::LeftParen))
            }
            ')' => {
                self.advance();
                Some(TokenKind::Delimiter(Delimiter::RightParen))
            }
            '{' => {
                self.advance();
                Some(TokenKind::Delimiter(Delimiter::LeftBrace))
            }
            '}' => {
                self.advance();
                Some(TokenKind::Delimiter(Delimiter::RightBrace))
            }
            '[' => {
                self.advance();
                Some(TokenKind::Delimiter(Delimiter::LeftBracket))
            }
            ']' => {
                self.advance();
                Some(TokenKind::Delimiter(Delimiter::RightBracket))
            }
            ';' => {
                self.advance();
                Some(TokenKind::Delimiter(Delimiter::Semicolon))
            }
            ',' => {
                self.advance();
                Some(TokenKind::Delimiter(Delimiter::Comma))
            }
            '"' => self.scan_string(),
            c if c.is_ascii_digit() => self.scan_number(),
            c if c.is_ascii_alphabetic() || c == '_' => self.scan_identifier_or_keyword(),
            _ => {
                self.advance();
                None
            }
        }
    }

    fn scan_string(&mut self) -> Option<TokenKind> {
        self.advance(); // consume opening quote
        let start = self.position;

        while !self.is_eof() && self.current_char() != Some('"') {
            if self.current_char() == Some('\\') {
                self.advance(); // consume backslash
                if !self.is_eof() {
                    self.advance(); // consume escaped character
                }
            } else {
                self.advance();
            }
        }

        if self.current_char() == Some('"') {
            let content = self.input[start..self.position].to_string();
            self.advance(); // consume closing quote
            Some(TokenKind::String(content))
        } else {
            None // unterminated string
        }
    }

    fn scan_number(&mut self) -> Option<TokenKind> {
        let start = self.position;

        while !self.is_eof() && self.current_char().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }

        let num_str = &self.input[start..self.position];
        num_str.parse().ok().map(TokenKind::Number)
    }

    fn scan_identifier_or_keyword(&mut self) -> Option<TokenKind> {
        let start = self.position;

        while !self.is_eof() {
            if let Some(c) = self.current_char() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let ident = &self.input[start..self.position];

        let token = match ident {
            "let" => TokenKind::Keyword(Keyword::Let),
            "if" => TokenKind::Keyword(Keyword::If),
            "else" => TokenKind::Keyword(Keyword::Else),
            "while" => TokenKind::Keyword(Keyword::While),
            "function" => TokenKind::Keyword(Keyword::Function),
            "return" => TokenKind::Keyword(Keyword::Return),
            _ => TokenKind::Identifier(ident.to_string()),
        };

        Some(token)
    }

    fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.current_char().is_some_and(|c| c.is_whitespace()) {
            self.advance();
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(c) = self.current_char() {
            self.position += c.len_utf8();
        }
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }
}

/// Span arithmetic demonstrations
pub fn demonstrate_span_arithmetic() {
    let start = ByteIndex::from(10);
    let offset = ByteOffset::from(5);

    // Adding offset to index
    let new_position = start + offset;
    assert_eq!(new_position, ByteIndex::from(15));

    // Subtracting offset from index
    let prev_position = new_position - offset;
    assert_eq!(prev_position, start);

    // Creating spans
    let span = Span::new(start, new_position);
    assert_eq!(span.start(), start);
    assert_eq!(span.end(), new_position);
}

/// Line offset calculations
pub fn demonstrate_line_offsets() {
    let line = LineIndex::from(5);
    let offset = LineOffset::from(3);

    // Moving forward by lines
    let new_line = line + offset;
    assert_eq!(new_line, LineIndex::from(8));

    // Moving backward by lines
    let prev_line = new_line - offset;
    assert_eq!(prev_line, line);
}

/// UTF-8 aware position tracking
pub fn track_utf8_positions(text: &str) -> Vec<(char, ByteIndex)> {
    let mut positions = Vec::new();
    let mut byte_pos = ByteIndex::from(0);

    for ch in text.chars() {
        positions.push((ch, byte_pos));
        byte_pos += ByteOffset::from(ch.len_utf8() as i64);
    }

    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_file() {
        let source = "let x = 42;\nlet y = x + 1;\nprint(y);";
        let file = SourceFile::new("test.lang".to_string(), source.to_string());

        // Test line index calculation
        assert_eq!(file.line_index(ByteIndex::from(0)), LineIndex::from(0));
        assert_eq!(file.line_index(ByteIndex::from(12)), LineIndex::from(1));
        assert_eq!(file.line_index(ByteIndex::from(27)), LineIndex::from(2));

        // Test column index calculation
        assert_eq!(file.column_index(ByteIndex::from(4)), ColumnIndex::from(4));
        assert_eq!(file.column_index(ByteIndex::from(16)), ColumnIndex::from(4));

        // Test location
        let loc = file.location(ByteIndex::from(16));
        assert_eq!(loc.line, LineIndex::from(1));
        assert_eq!(loc.column, ColumnIndex::from(4));
    }

    #[test]
    fn test_span_manager() {
        let mut manager = SpanManager::new();

        let file1 = manager.add_file("main.lang".to_string(), "let x = 10;".to_string());

        let file2 = manager.add_file(
            "lib.lang".to_string(),
            "function add(a, b) { return a + b; }".to_string(),
        );

        assert!(manager.get_file(file1).is_some());
        assert!(manager.get_file(file2).is_some());
        assert_eq!(manager.find_file("main.lang"), Some(file1));
        assert_eq!(manager.find_file("lib.lang"), Some(file2));
    }

    #[test]
    fn test_lexer() {
        let mut manager = SpanManager::new();
        let file_id = manager.add_file("test.lang".to_string(), "let x = 42 + 3;".to_string());

        let mut lexer = Lexer::new("let x = 42 + 3;".to_string(), file_id);
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 7);

        // Check first token (let)
        assert_eq!(tokens[0].kind, TokenKind::Keyword(Keyword::Let));
        assert_eq!(tokens[0].span.start(), ByteIndex::from(0));
        assert_eq!(tokens[0].span.end(), ByteIndex::from(3));

        // Check identifier
        assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));

        // Check equals operator
        assert_eq!(tokens[2].kind, TokenKind::Operator(Operator::Assign));

        // Check number
        assert_eq!(tokens[3].kind, TokenKind::Number(42));

        // Check plus operator
        assert_eq!(tokens[4].kind, TokenKind::Operator(Operator::Plus));

        // Check second number
        assert_eq!(tokens[5].kind, TokenKind::Number(3));

        // Check semicolon
        assert_eq!(tokens[6].kind, TokenKind::Delimiter(Delimiter::Semicolon));
    }

    #[test]
    fn test_span_arithmetic() {
        demonstrate_span_arithmetic();
    }

    #[test]
    fn test_line_offsets() {
        demonstrate_line_offsets();
    }

    #[test]
    fn test_utf8_tracking() {
        let text = "hello 世界!";
        let positions = track_utf8_positions(text);

        // ASCII characters take 1 byte
        assert_eq!(positions[0], ('h', ByteIndex::from(0)));
        assert_eq!(positions[5], (' ', ByteIndex::from(5)));

        // Chinese characters take 3 bytes each
        assert_eq!(positions[6], ('世', ByteIndex::from(6)));
        assert_eq!(positions[7], ('界', ByteIndex::from(9)));
        assert_eq!(positions[8], ('!', ByteIndex::from(12)));
    }

    #[test]
    fn test_span_merging() {
        let manager = SpanManager::new();

        let span1 = manager.create_span(ByteIndex::from(10), ByteIndex::from(20));
        let span2 = manager.create_span(ByteIndex::from(15), ByteIndex::from(25));

        let merged = manager.merge_spans(span1, span2);
        assert_eq!(merged.start(), ByteIndex::from(10));
        assert_eq!(merged.end(), ByteIndex::from(25));
    }
}
