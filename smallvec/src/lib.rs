use smallvec::{smallvec, SmallVec};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Number(i64),
    Operator(char),
    Keyword(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub type TokenStream = SmallVec<[Token; 32]>;

pub fn tokenize_expression(input: &str) -> TokenStream {
    let mut tokens = SmallVec::new();
    let mut chars = input.char_indices().peekable();

    while let Some((i, ch)) = chars.next() {
        match ch {
            ' ' | '\t' | '\n' => continue,
            '+' | '-' | '*' | '/' | '(' | ')' => {
                tokens.push(Token {
                    kind: TokenKind::Operator(ch),
                    span: Span {
                        start: i,
                        end: i + 1,
                    },
                });
            }
            '0'..='9' => {
                let start = i;
                let mut value = ch.to_digit(10).unwrap() as i64;

                while let Some(&(_, ch)) = chars.peek() {
                    if ch.is_ascii_digit() {
                        value = value * 10 + ch.to_digit(10).unwrap() as i64;
                        chars.next();
                    } else {
                        break;
                    }
                }

                let end = chars.peek().map(|&(i, _)| i).unwrap_or(input.len());
                tokens.push(Token {
                    kind: TokenKind::Number(value),
                    span: Span { start, end },
                });
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = i;
                let mut ident = String::new();
                ident.push(ch);

                while let Some(&(_, ch)) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                let end = chars.peek().map(|&(i, _)| i).unwrap_or(input.len());
                let kind = match ident.as_str() {
                    "if" | "else" | "while" | "for" | "return" => TokenKind::Keyword(ident),
                    _ => TokenKind::Identifier(ident),
                };

                tokens.push(Token {
                    kind,
                    span: Span { start, end },
                });
            }
            _ => {}
        }
    }

    tokens
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub kind: AstKind,
    pub children: SmallVec<[Box<AstNode>; 4]>,
}

#[derive(Debug, Clone)]
pub enum AstKind {
    Program,
    Function(String),
    Block,
    Expression,
    Statement,
    Identifier(String),
    Number(i64),
}

pub fn build_simple_ast() -> AstNode {
    AstNode {
        kind: AstKind::Program,
        children: smallvec![Box::new(AstNode {
            kind: AstKind::Function("main".to_string()),
            children: smallvec![Box::new(AstNode {
                kind: AstKind::Block,
                children: smallvec![Box::new(AstNode {
                    kind: AstKind::Expression,
                    children: smallvec![Box::new(AstNode {
                        kind: AstKind::Number(42),
                        children: SmallVec::new(),
                    })],
                })],
            })],
        })],
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operands: SmallVec<[Operand; 3]>,
}

#[derive(Debug, Clone)]
pub enum Opcode {
    Load,
    Store,
    Add,
    Sub,
    Mul,
    Jmp,
    Ret,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(u8),
    Immediate(i32),
    Memory(u32),
}

pub fn create_instruction_sequence() -> Vec<Instruction> {
    vec![
        Instruction {
            opcode: Opcode::Load,
            operands: smallvec![Operand::Register(0), Operand::Memory(0x1000)],
        },
        Instruction {
            opcode: Opcode::Load,
            operands: smallvec![Operand::Register(1), Operand::Memory(0x1004)],
        },
        Instruction {
            opcode: Opcode::Add,
            operands: smallvec![
                Operand::Register(2),
                Operand::Register(0),
                Operand::Register(1)
            ],
        },
        Instruction {
            opcode: Opcode::Store,
            operands: smallvec![Operand::Memory(0x1008), Operand::Register(2)],
        },
        Instruction {
            opcode: Opcode::Ret,
            operands: SmallVec::new(),
        },
    ]
}

pub fn demonstrate_capacity() {
    let mut vec: SmallVec<[i32; 4]> = SmallVec::new();

    println!("Initial capacity: {}", vec.inline_size());
    println!("Is heap allocated: {}", vec.spilled());

    for i in 0..6 {
        vec.push(i);
        println!(
            "After pushing {}: capacity = {}, spilled = {}",
            i,
            vec.capacity(),
            vec.spilled()
        );
    }
}

pub struct SymbolTable {
    scopes: SmallVec<[Scope; 8]>,
}

pub struct Scope {
    symbols: SmallVec<[(String, SymbolInfo); 16]>,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub kind: SymbolKind,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable,
    Function,
    Parameter,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: smallvec![Scope {
                symbols: SmallVec::new(),
            }],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope {
            symbols: SmallVec::new(),
        });
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn insert(&mut self, name: String, info: SymbolInfo) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.symbols.push((name, info));
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            for (sym_name, info) in &scope.symbols {
                if sym_name == name {
                    return Some(info);
                }
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct CompactError {
    pub messages: SmallVec<[String; 2]>,
    pub locations: SmallVec<[Location; 2]>,
}

#[derive(Debug)]
pub struct Location {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl CompactError {
    pub fn new(message: String, location: Location) -> Self {
        Self {
            messages: smallvec![message],
            locations: smallvec![location],
        }
    }

    pub fn add_context(&mut self, message: String, location: Location) {
        self.messages.push(message);
        self.locations.push(location);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenization() {
        let tokens = tokenize_expression("x + 42 * (y - 3)");
        assert_eq!(tokens.len(), 9);
        assert!(matches!(tokens[0].kind, TokenKind::Identifier(_)));
        assert!(matches!(tokens[2].kind, TokenKind::Number(42)));
    }

    #[test]
    fn test_inline_capacity() {
        let vec: SmallVec<[i32; 8]> = smallvec![1, 2, 3, 4];
        assert!(!vec.spilled());
        assert_eq!(vec.len(), 4);
        assert_eq!(vec.capacity(), 8);
    }

    #[test]
    fn test_spill_to_heap() {
        let mut vec: SmallVec<[i32; 2]> = SmallVec::new();
        vec.push(1);
        vec.push(2);
        assert!(!vec.spilled());

        vec.push(3);
        assert!(vec.spilled());
        assert!(vec.capacity() >= 3);
    }

    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();

        table.insert(
            "x".to_string(),
            SymbolInfo {
                kind: SymbolKind::Variable,
                offset: 0,
            },
        );

        table.push_scope();
        table.insert(
            "y".to_string(),
            SymbolInfo {
                kind: SymbolKind::Variable,
                offset: 4,
            },
        );

        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_some());

        table.pop_scope();
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_none());
    }
}
