use rowan::{GreenNode, GreenNodeBuilder, Language, SyntaxNode, SyntaxToken, TextRange, TextSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    Whitespace = 0,
    Comment,
    Ident,
    Number,
    String,
    Plus,
    Minus,
    Star,
    Slash,
    Eq,
    Neq,
    Lt,
    Gt,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,
    Keyword,
    Error,
    Root,
    BinaryExpr,
    UnaryExpr,
    ParenExpr,
    Literal,
    BlockStmt,
    ExprStmt,
    LetStmt,
    IfStmt,
    WhileStmt,
    ReturnStmt,
    FnDef,
    ParamList,
    TypeRef,
    Path,
    CallExpr,
    ArgList,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {}

impl Language for Lang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::ArgList as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNodeRef = SyntaxNode<Lang>;
pub type SyntaxTokenRef = SyntaxToken<Lang>;

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub green_node: GreenNode,
    pub errors: Vec<ParseError>,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub range: TextRange,
}

pub struct Parser {
    builder: GreenNodeBuilder<'static>,
    errors: Vec<ParseError>,
    tokens: Vec<Token>,
    cursor: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: SyntaxKind,
    pub text: String,
    pub offset: TextSize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
            tokens,
            cursor: 0,
        }
    }

    pub fn parse(mut self) -> ParseResult {
        self.builder.start_node(SyntaxKind::Root.into());

        while !self.at_end() {
            if self.at(SyntaxKind::Whitespace) || self.at(SyntaxKind::Comment) {
                self.trivia();
            } else {
                self.statement();
            }
        }

        self.builder.finish_node();

        ParseResult {
            green_node: self.builder.finish(),
            errors: self.errors,
        }
    }

    fn statement(&mut self) {
        match self.current_kind() {
            Some(SyntaxKind::Keyword) if self.current_text() == Some("let") => {
                self.let_statement();
            }
            Some(SyntaxKind::Keyword) if self.current_text() == Some("if") => {
                self.if_statement();
            }
            Some(SyntaxKind::Keyword) if self.current_text() == Some("while") => {
                self.while_statement();
            }
            Some(SyntaxKind::Keyword) if self.current_text() == Some("return") => {
                self.return_statement();
            }
            Some(SyntaxKind::Keyword) if self.current_text() == Some("fn") => {
                self.function_definition();
            }
            _ => {
                self.expression_statement();
            }
        }
    }

    fn let_statement(&mut self) {
        self.builder.start_node(SyntaxKind::LetStmt.into());
        self.consume(SyntaxKind::Keyword);
        self.skip_trivia();
        self.consume(SyntaxKind::Ident);
        self.skip_trivia();

        if self.at(SyntaxKind::Eq) {
            self.consume(SyntaxKind::Eq);
            self.skip_trivia();
            self.expression();
        }

        self.skip_trivia();
        self.consume(SyntaxKind::Semicolon);
        self.builder.finish_node();
    }

    fn if_statement(&mut self) {
        self.builder.start_node(SyntaxKind::IfStmt.into());
        self.consume(SyntaxKind::Keyword);
        self.skip_trivia();
        self.expression();
        self.skip_trivia();
        self.block();

        if self.at_keyword("else") {
            self.consume(SyntaxKind::Keyword);
            self.skip_trivia();
            if self.at_keyword("if") {
                self.if_statement();
            } else {
                self.block();
            }
        }

        self.builder.finish_node();
    }

    fn while_statement(&mut self) {
        self.builder.start_node(SyntaxKind::WhileStmt.into());
        self.consume(SyntaxKind::Keyword);
        self.skip_trivia();
        self.expression();
        self.skip_trivia();
        self.block();
        self.builder.finish_node();
    }

    fn return_statement(&mut self) {
        self.builder.start_node(SyntaxKind::ReturnStmt.into());
        self.consume(SyntaxKind::Keyword);
        self.skip_trivia();

        if !self.at(SyntaxKind::Semicolon) {
            self.expression();
        }

        self.skip_trivia();
        self.consume(SyntaxKind::Semicolon);
        self.builder.finish_node();
    }

    fn function_definition(&mut self) {
        self.builder.start_node(SyntaxKind::FnDef.into());
        self.consume(SyntaxKind::Keyword);
        self.skip_trivia();
        self.consume(SyntaxKind::Ident);
        self.skip_trivia();
        self.parameter_list();
        self.skip_trivia();
        self.block();
        self.builder.finish_node();
    }

    fn parameter_list(&mut self) {
        self.builder.start_node(SyntaxKind::ParamList.into());
        self.consume(SyntaxKind::LParen);
        self.skip_trivia();

        if !self.at(SyntaxKind::RParen) {
            loop {
                self.consume(SyntaxKind::Ident);
                self.skip_trivia();

                if self.at(SyntaxKind::Comma) {
                    self.consume(SyntaxKind::Comma);
                    self.skip_trivia();
                } else {
                    break;
                }
            }
        }

        self.consume(SyntaxKind::RParen);
        self.builder.finish_node();
    }

    fn block(&mut self) {
        self.builder.start_node(SyntaxKind::BlockStmt.into());
        self.consume(SyntaxKind::LBrace);

        while !self.at_end() && !self.at(SyntaxKind::RBrace) {
            if self.at(SyntaxKind::Whitespace) || self.at(SyntaxKind::Comment) {
                self.trivia();
            } else {
                self.statement();
            }
        }

        self.consume(SyntaxKind::RBrace);
        self.builder.finish_node();
    }

    fn expression_statement(&mut self) {
        self.builder.start_node(SyntaxKind::ExprStmt.into());
        self.expression();
        self.skip_trivia();
        self.consume(SyntaxKind::Semicolon);
        self.builder.finish_node();
    }

    fn expression(&mut self) {
        self.binary_expression(0);
    }

    fn binary_expression(&mut self, min_precedence: u8) {
        self.unary_expression();

        // Include whitespace in the tree
        while self.at(SyntaxKind::Whitespace) {
            self.trivia();
        }

        while let Some(op_precedence) = self.current_binary_op_precedence() {
            if op_precedence < min_precedence {
                break;
            }

            let checkpoint = self.builder.checkpoint();

            if let Some(
                k @ (SyntaxKind::Plus
                | SyntaxKind::Minus
                | SyntaxKind::Star
                | SyntaxKind::Slash
                | SyntaxKind::Eq
                | SyntaxKind::Neq
                | SyntaxKind::Lt
                | SyntaxKind::Gt),
            ) = self.current_kind()
            {
                self.consume(k);
            }

            // Include whitespace in the tree
            while self.at(SyntaxKind::Whitespace) {
                self.trivia();
            }

            self.binary_expression(op_precedence + 1);

            self.builder
                .start_node_at(checkpoint, SyntaxKind::BinaryExpr.into());
            self.builder.finish_node();
        }
    }

    fn unary_expression(&mut self) {
        if self.at(SyntaxKind::Minus) || self.at(SyntaxKind::Plus) {
            self.builder.start_node(SyntaxKind::UnaryExpr.into());
            self.consume(self.current_kind().unwrap());
            while self.at(SyntaxKind::Whitespace) {
                self.trivia();
            }
            self.unary_expression();
            self.builder.finish_node();
        } else {
            self.postfix_expression();
        }
    }

    fn postfix_expression(&mut self) {
        self.primary_expression();

        while self.at(SyntaxKind::LParen) {
            self.builder.start_node(SyntaxKind::CallExpr.into());
            let checkpoint = self.builder.checkpoint();
            self.argument_list();
            self.builder
                .start_node_at(checkpoint, SyntaxKind::CallExpr.into());
            self.builder.finish_node();
        }
    }

    fn argument_list(&mut self) {
        self.builder.start_node(SyntaxKind::ArgList.into());
        self.consume(SyntaxKind::LParen);
        self.skip_trivia();

        if !self.at(SyntaxKind::RParen) {
            loop {
                self.expression();
                self.skip_trivia();

                if self.at(SyntaxKind::Comma) {
                    self.consume(SyntaxKind::Comma);
                    self.skip_trivia();
                } else {
                    break;
                }
            }
        }

        self.consume(SyntaxKind::RParen);
        self.builder.finish_node();
    }

    fn primary_expression(&mut self) {
        match self.current_kind() {
            Some(SyntaxKind::Number) => {
                self.builder.start_node(SyntaxKind::Literal.into());
                self.consume(SyntaxKind::Number);
                self.builder.finish_node();
            }
            Some(SyntaxKind::String) => {
                self.builder.start_node(SyntaxKind::Literal.into());
                self.consume(SyntaxKind::String);
                self.builder.finish_node();
            }
            Some(SyntaxKind::Ident) => {
                self.consume(SyntaxKind::Ident);
            }
            Some(SyntaxKind::LParen) => {
                self.builder.start_node(SyntaxKind::ParenExpr.into());
                self.consume(SyntaxKind::LParen);
                while self.at(SyntaxKind::Whitespace) {
                    self.trivia();
                }
                self.expression();
                while self.at(SyntaxKind::Whitespace) {
                    self.trivia();
                }
                self.consume(SyntaxKind::RParen);
                self.builder.finish_node();
            }
            _ => {
                self.error("Expected expression");
                self.advance();
            }
        }
    }

    fn current_binary_op_precedence(&self) -> Option<u8> {
        match self.current_kind()? {
            SyntaxKind::Star | SyntaxKind::Slash => Some(5),
            SyntaxKind::Plus | SyntaxKind::Minus => Some(4),
            SyntaxKind::Lt | SyntaxKind::Gt => Some(3),
            SyntaxKind::Eq | SyntaxKind::Neq => Some(2),
            _ => None,
        }
    }

    fn trivia(&mut self) {
        while self.at(SyntaxKind::Whitespace) || self.at(SyntaxKind::Comment) {
            let kind = self.current_kind().unwrap();
            self.consume(kind);
        }
    }

    fn skip_trivia(&mut self) {
        while self.at(SyntaxKind::Whitespace) || self.at(SyntaxKind::Comment) {
            self.advance();
        }
    }

    fn consume(&mut self, expected: SyntaxKind) {
        if self.at(expected) {
            let token = &self.tokens[self.cursor];
            self.builder.token(expected.into(), &token.text);
            self.advance();
        } else {
            self.error(&format!("Expected {:?}", expected));
        }
    }

    fn at(&self, kind: SyntaxKind) -> bool {
        self.current_kind() == Some(kind)
    }

    fn at_keyword(&self, keyword: &str) -> bool {
        self.at(SyntaxKind::Keyword) && self.current_text() == Some(keyword)
    }

    fn current_kind(&self) -> Option<SyntaxKind> {
        self.tokens.get(self.cursor).map(|t| t.kind)
    }

    fn current_text(&self) -> Option<&str> {
        self.tokens.get(self.cursor).map(|t| t.text.as_str())
    }

    fn advance(&mut self) {
        if self.cursor < self.tokens.len() {
            self.cursor += 1;
        }
    }

    fn at_end(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn error(&mut self, message: &str) {
        let offset = self
            .tokens
            .get(self.cursor)
            .map(|t| t.offset)
            .unwrap_or_else(|| TextSize::from(0));

        self.errors.push(ParseError {
            message: message.to_string(),
            range: TextRange::empty(offset),
        });
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut offset = TextSize::from(0);
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        let start = offset;
        offset += TextSize::of(ch);

        let (kind, text) = match ch {
            ' ' | '\t' | '\n' | '\r' => {
                let mut text = String::from(ch);
                while let Some(&next) = chars.peek() {
                    if next.is_whitespace() {
                        text.push(next);
                        offset += TextSize::of(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                (SyntaxKind::Whitespace, text)
            }
            '/' if chars.peek() == Some(&'/') => {
                chars.next();
                offset += TextSize::of('/');
                let mut text = String::from("//");
                while let Some(&next) = chars.peek() {
                    if next != '\n' {
                        text.push(next);
                        offset += TextSize::of(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                (SyntaxKind::Comment, text)
            }
            '+' => (SyntaxKind::Plus, String::from("+")),
            '-' => (SyntaxKind::Minus, String::from("-")),
            '*' => (SyntaxKind::Star, String::from("*")),
            '/' => (SyntaxKind::Slash, String::from("/")),
            '=' if chars.peek() == Some(&'=') => {
                chars.next();
                offset += TextSize::of('=');
                (SyntaxKind::Eq, String::from("=="))
            }
            '=' => (SyntaxKind::Eq, String::from("=")),
            '!' if chars.peek() == Some(&'=') => {
                chars.next();
                offset += TextSize::of('=');
                (SyntaxKind::Neq, String::from("!="))
            }
            '<' => (SyntaxKind::Lt, String::from("<")),
            '>' => (SyntaxKind::Gt, String::from(">")),
            '(' => (SyntaxKind::LParen, String::from("(")),
            ')' => (SyntaxKind::RParen, String::from(")")),
            '{' => (SyntaxKind::LBrace, String::from("{")),
            '}' => (SyntaxKind::RBrace, String::from("}")),
            ';' => (SyntaxKind::Semicolon, String::from(";")),
            ',' => (SyntaxKind::Comma, String::from(",")),
            '"' => {
                let mut text = String::from("\"");
                while let Some(&next) = chars.peek() {
                    text.push(next);
                    offset += TextSize::of(next);
                    chars.next();
                    if next == '"' {
                        break;
                    }
                }
                (SyntaxKind::String, text)
            }
            c if c.is_ascii_digit() => {
                let mut text = String::from(c);
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() {
                        text.push(next);
                        offset += TextSize::of(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                (SyntaxKind::Number, text)
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut text = String::from(c);
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_alphanumeric() || next == '_' {
                        text.push(next);
                        offset += TextSize::of(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let kind = match text.as_str() {
                    "let" | "if" | "else" | "while" | "for" | "fn" | "return" | "true"
                    | "false" | "struct" | "enum" | "impl" => SyntaxKind::Keyword,
                    _ => SyntaxKind::Ident,
                };
                (kind, text)
            }
            _ => (SyntaxKind::Error, String::from(ch)),
        };

        tokens.push(Token {
            kind,
            text,
            offset: start,
        });
    }

    tokens
}

#[derive(Debug)]
pub struct IncrementalReparser {
    _old_tree: SyntaxNodeRef,
    edits: Vec<TextEdit>,
}

#[derive(Debug, Clone)]
pub struct TextEdit {
    pub range: TextRange,
    pub new_text: String,
}

impl IncrementalReparser {
    pub fn new(tree: SyntaxNodeRef) -> Self {
        Self {
            _old_tree: tree,
            edits: Vec::new(),
        }
    }

    pub fn add_edit(&mut self, edit: TextEdit) {
        self.edits.push(edit);
    }

    pub fn reparse(&self, new_text: &str) -> ParseResult {
        let tokens = tokenize(new_text);
        let parser = Parser::new(tokens);
        parser.parse()
    }
}

pub struct SyntaxTreeBuilder {
    green: GreenNode,
}

impl SyntaxTreeBuilder {
    pub fn new(green: GreenNode) -> Self {
        Self { green }
    }

    pub fn build(self) -> SyntaxNodeRef {
        SyntaxNodeRef::new_root(self.green)
    }
}

pub fn parse_expression(input: &str) -> SyntaxNodeRef {
    let tokens = tokenize(input);
    let mut parser = Parser::new(tokens);

    // Build just an expression tree
    parser.builder.start_node(SyntaxKind::Root.into());

    // Include leading whitespace as trivia
    while parser.at(SyntaxKind::Whitespace) {
        parser.trivia();
    }

    // Parse the expression
    if !parser.at_end() {
        parser.expression();
    }

    // Include trailing whitespace as trivia
    while parser.at(SyntaxKind::Whitespace) {
        parser.trivia();
    }

    parser.builder.finish_node();
    let green_node = parser.builder.finish();

    SyntaxTreeBuilder::new(green_node).build()
}

pub struct AstNode {
    syntax: SyntaxNodeRef,
}

impl AstNode {
    pub fn cast(syntax: SyntaxNodeRef) -> Option<Self> {
        match syntax.kind() {
            SyntaxKind::Root
            | SyntaxKind::BinaryExpr
            | SyntaxKind::UnaryExpr
            | SyntaxKind::ParenExpr
            | SyntaxKind::Literal
            | SyntaxKind::BlockStmt
            | SyntaxKind::ExprStmt
            | SyntaxKind::LetStmt
            | SyntaxKind::IfStmt
            | SyntaxKind::WhileStmt
            | SyntaxKind::ReturnStmt
            | SyntaxKind::FnDef
            | SyntaxKind::CallExpr => Some(Self { syntax }),
            _ => None,
        }
    }

    pub fn syntax(&self) -> &SyntaxNodeRef {
        &self.syntax
    }
}

pub trait AstToken {
    fn cast(syntax: SyntaxTokenRef) -> Option<Self>
    where
        Self: Sized;
    fn syntax(&self) -> &SyntaxTokenRef;
}

pub struct Identifier {
    syntax: SyntaxTokenRef,
}

impl AstToken for Identifier {
    fn cast(syntax: SyntaxTokenRef) -> Option<Self> {
        if syntax.kind() == SyntaxKind::Ident {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxTokenRef {
        &self.syntax
    }
}

impl Identifier {
    pub fn text(&self) -> &str {
        self.syntax.text()
    }
}

pub fn walk_tree(node: &SyntaxNodeRef, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}{:?}", indent, node.kind());

    for child in node.children() {
        walk_tree(&child, depth + 1);
    }
}

pub fn find_node_at_offset(root: &SyntaxNodeRef, offset: TextSize) -> Option<SyntaxNodeRef> {
    if !root.text_range().contains(offset) {
        return None;
    }

    let mut result = root.clone();

    for child in root.children() {
        if child.text_range().contains(offset) {
            if let Some(deeper) = find_node_at_offset(&child, offset) {
                result = deeper;
            }
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenization() {
        let input = "let x = 42 + y;";
        let tokens = tokenize(input);

        assert_eq!(tokens[0].kind, SyntaxKind::Keyword);
        assert_eq!(tokens[0].text, "let");

        assert_eq!(tokens[2].kind, SyntaxKind::Ident);
        assert_eq!(tokens[2].text, "x");
    }

    #[test]
    fn test_parse_expression() {
        let input = "x + 42 * (y - 3)";

        // First check tokenization
        let tokens = tokenize(input);

        // The expression should tokenize properly
        assert!(tokens.len() > 5, "Expected more tokens, got: {:?}", tokens);

        // Check if tokens include operators
        let has_plus = tokens.iter().any(|t| t.kind == SyntaxKind::Plus);
        let has_star = tokens.iter().any(|t| t.kind == SyntaxKind::Star);
        assert!(has_plus, "Missing + operator in tokens: {:?}", tokens);
        assert!(has_star, "Missing * operator in tokens: {:?}", tokens);

        let tree = parse_expression(input);

        assert_eq!(tree.kind(), SyntaxKind::Root);

        // Check the tree contains the full expression
        let tree_text = tree.text().to_string();
        assert_eq!(
            tree_text.trim(),
            input,
            "Tree text doesn't match input. Got '{}' expected '{}'",
            tree_text.trim(),
            input
        );
    }

    #[test]
    fn test_incremental_reparse() {
        let input = "let x = 42;";
        let tree = parse_expression(input);

        let mut reparser = IncrementalReparser::new(tree);
        reparser.add_edit(TextEdit {
            range: TextRange::new(TextSize::from(8), TextSize::from(10)),
            new_text: "100".to_string(),
        });

        let new_tree = reparser.reparse("let x = 100;");
        assert_eq!(new_tree.errors.len(), 0);
    }

    #[test]
    fn test_ast_node_cast() {
        let input = "42 + x";
        let tree = parse_expression(input);

        if let Some(first_child) = tree.first_child() {
            let ast_node = AstNode::cast(first_child);
            assert!(ast_node.is_some());
        }
    }
}
