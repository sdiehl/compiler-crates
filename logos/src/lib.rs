use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\f]+")] // Skip whitespace except newlines
pub enum Token {
    // Keywords
    #[token("fn")]
    Function,
    #[token("let")]
    Let,
    #[token("const")]
    Const,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("return")]
    Return,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("impl")]
    Impl,
    #[token("trait")]
    Trait,
    #[token("pub")]
    Pub,
    #[token("mod")]
    Mod,
    #[token("use")]
    Use,
    #[token("mut")]
    Mut,
    #[token("true")]
    True,
    #[token("false")]
    False,

    // Identifiers and literals
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(Option<i64>),

    #[regex(r"-?[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(Option<f64>),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    String(String),

    #[regex(r"'([^'\\]|\\.)'")]
    Char,

    // Comments
    #[regex(r"//[^\n]*", logos::skip, allow_greedy = true)]
    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip)]
    Comment,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("=")]
    Assign,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("<<")]
    LeftShift,
    #[token(">>")]
    RightShift,
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    StarAssign,
    #[token("/=")]
    SlashAssign,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("::")]
    PathSeparator,

    // Punctuation
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("..")]
    DotDot,
    #[token("...")]
    DotDotDot,
    #[token("?")]
    Question,

    // Special handling for newlines (for line counting)
    #[token("\n")]
    Newline,
}

pub struct TokenStream<'source> {
    lexer: Lexer<'source, Token>,
    peeked: Option<Result<Token, ()>>,
}

impl<'source> TokenStream<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            lexer: Token::lexer(source),
            peeked: None,
        }
    }

    pub fn next_token(&mut self) -> Option<Result<Token, ()>> {
        if let Some(token) = self.peeked.take() {
            return Some(token);
        }
        self.lexer.next()
    }

    pub fn peek_token(&mut self) -> Option<&Result<Token, ()>> {
        if self.peeked.is_none() {
            self.peeked = self.lexer.next();
        }
        self.peeked.as_ref()
    }

    pub fn span(&self) -> std::ops::Range<usize> {
        self.lexer.span()
    }

    pub fn slice(&self) -> &'source str {
        self.lexer.slice()
    }

    pub fn remainder(&self) -> &'source str {
        self.lexer.remainder()
    }
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}

pub struct SourceTracker<'source> {
    source: &'source str,
    line_starts: Vec<usize>,
}

impl<'source> SourceTracker<'source> {
    pub fn new(source: &'source str) -> Self {
        let mut line_starts = vec![0];
        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }
        Self {
            source,
            line_starts,
        }
    }

    pub fn location(&self, byte_offset: usize) -> SourceLocation {
        let line = self
            .line_starts
            .binary_search(&byte_offset)
            .unwrap_or_else(|i| i.saturating_sub(1));

        let line_start = self.line_starts[line];
        let column = self.source[line_start..byte_offset].chars().count();

        SourceLocation {
            line: line + 1,     // 1-based
            column: column + 1, // 1-based
            byte_offset,
        }
    }

    pub fn line_content(&self, line: usize) -> &'source str {
        if line == 0 || line > self.line_starts.len() {
            return "";
        }

        let start = self.line_starts[line - 1];
        let end = if line < self.line_starts.len() {
            self.line_starts[line] - 1
        } else {
            self.source.len()
        };

        &self.source[start..end]
    }
}

pub fn tokenize(input: &str) -> Vec<TokenSpan> {
    let mut tokens = Vec::new();
    let mut lexer = Token::lexer(input);

    while let Some(result) = lexer.next() {
        if let Ok(token) = result {
            tokens.push((token, lexer.span()));
        }
    }

    tokens
}

pub type TokenSpan = (Token, std::ops::Range<usize>);
pub type ErrorSpan = std::ops::Range<usize>;

pub fn tokenize_with_errors(input: &str) -> (Vec<TokenSpan>, Vec<ErrorSpan>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut lexer = Token::lexer(input);

    while let Some(result) = lexer.next() {
        match result {
            Ok(token) => tokens.push((token, lexer.span())),
            Err(()) => errors.push(lexer.span()),
        }
    }

    (tokens, errors)
}

// Example: Building a simple expression lexer with custom state
#[derive(Logos, Debug, PartialEq)]
pub enum ExprToken {
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i32>().ok())]
    Number(Option<i32>),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Times,
    #[token("/")]
    Divide,
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,

    #[regex(r"[ \t\n]+", logos::skip)]
    Whitespace,
}

pub fn parse_expression(input: &str) -> Vec<ExprToken> {
    ExprToken::lexer(input).filter_map(Result::ok).collect()
}

// Advanced: Lexer with extras for tracking indentation
#[derive(Logos, Debug, PartialEq)]
#[logos(extras = IndentationTracker)]
pub enum IndentedToken {
    #[token("\n", |lex| {
        lex.extras.newline();
        logos::Skip
    })]
    Newline,

    #[regex(r"[ ]+", |lex| {
        let spaces = lex.slice().len();
        lex.extras.track_indent(spaces)
    })]
    Indent(IndentLevel),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[token(":")]
    Colon,

    #[regex(r"#[^\n]*", logos::skip, allow_greedy = true)]
    Comment,
}

#[derive(Default)]
pub struct IndentationTracker {
    at_line_start: bool,
    _current_indent: usize,
    indent_stack: Vec<usize>,
}

impl IndentationTracker {
    fn newline(&mut self) {
        self.at_line_start = true;
    }

    fn track_indent(&mut self, spaces: usize) -> IndentLevel {
        if !self.at_line_start {
            return IndentLevel::None;
        }

        self.at_line_start = false;

        if self.indent_stack.is_empty() {
            self.indent_stack.push(0);
        }

        let previous = *self.indent_stack.last().unwrap();

        use std::cmp::Ordering;
        match spaces.cmp(&previous) {
            Ordering::Greater => {
                self.indent_stack.push(spaces);
                IndentLevel::Indent
            }
            Ordering::Less => {
                let mut dedent_count = 0;
                while let Some(&level) = self.indent_stack.last() {
                    if level <= spaces {
                        break;
                    }
                    self.indent_stack.pop();
                    dedent_count += 1;
                }
                IndentLevel::Dedent(dedent_count)
            }
            Ordering::Equal => IndentLevel::None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum IndentLevel {
    None,
    Indent,
    Dedent(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let input = "fn main() { let x = 42; }";
        let tokens = tokenize(input);

        assert_eq!(tokens[0].0, Token::Function);
        assert_eq!(tokens[1].0, Token::Identifier("main".to_string()));
        assert_eq!(tokens[2].0, Token::LeftParen);
        assert_eq!(tokens[3].0, Token::RightParen);
        assert_eq!(tokens[4].0, Token::LeftBrace);
        assert_eq!(tokens[5].0, Token::Let);
    }

    #[test]
    fn test_numeric_literals() {
        let input = "42 -17 3.14 -2.718";
        let tokens = tokenize(input);

        assert_eq!(tokens[0].0, Token::Integer(Some(42)));
        assert_eq!(tokens[1].0, Token::Integer(Some(-17)));
        assert_eq!(tokens[2].0, Token::Float(Some(3.14)));
        assert_eq!(tokens[3].0, Token::Float(Some(-2.718)));
    }

    #[test]
    fn test_string_literals() {
        let input = r#""hello" "world\n" "with\"quotes\"""#;
        let tokens = tokenize(input);

        assert_eq!(tokens[0].0, Token::String("hello".to_string()));
        assert_eq!(tokens[1].0, Token::String("world\\n".to_string()));
        assert_eq!(tokens[2].0, Token::String("with\\\"quotes\\\"".to_string()));
    }

    #[test]
    fn test_operators() {
        let input = "+ - * / == != <= >= && || -> =>";
        let tokens = tokenize(input);

        assert_eq!(tokens[0].0, Token::Plus);
        assert_eq!(tokens[1].0, Token::Minus);
        assert_eq!(tokens[2].0, Token::Star);
        assert_eq!(tokens[3].0, Token::Slash);
        assert_eq!(tokens[4].0, Token::Equal);
        assert_eq!(tokens[5].0, Token::NotEqual);
        assert_eq!(tokens[6].0, Token::LessEqual);
        assert_eq!(tokens[7].0, Token::GreaterEqual);
        assert_eq!(tokens[8].0, Token::And);
        assert_eq!(tokens[9].0, Token::Or);
        assert_eq!(tokens[10].0, Token::Arrow);
        assert_eq!(tokens[11].0, Token::FatArrow);
    }

    #[test]
    fn test_error_handling() {
        let input = "let x = 42 @ invalid";
        let (_tokens, errors) = tokenize_with_errors(input);

        assert!(!errors.is_empty());
        assert_eq!(errors[0], 11..12); // Position of '@'
    }

    #[test]
    fn test_source_location() {
        let input = "fn main() {\n    let x = 42;\n}";
        let tracker = SourceTracker::new(input);

        // 'l' in 'let' on line 2
        let loc = tracker.location(16);
        assert_eq!(loc.line, 2);
        assert_eq!(loc.column, 5);

        // Get line content
        let line2 = tracker.line_content(2);
        assert_eq!(line2, "    let x = 42;");
    }

    #[test]
    fn test_expression_lexer() {
        let input = "x + 42 * (y - 3)";
        let tokens = parse_expression(input);

        assert_eq!(tokens[0], ExprToken::Identifier("x".to_string()));
        assert_eq!(tokens[1], ExprToken::Plus);
        assert_eq!(tokens[2], ExprToken::Number(Some(42)));
        assert_eq!(tokens[3], ExprToken::Times);
        assert_eq!(tokens[4], ExprToken::LeftParen);
        assert_eq!(tokens[5], ExprToken::Identifier("y".to_string()));
        assert_eq!(tokens[6], ExprToken::Minus);
        assert_eq!(tokens[7], ExprToken::Number(Some(3)));
        assert_eq!(tokens[8], ExprToken::RightParen);
    }
}
