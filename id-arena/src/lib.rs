use std::collections::HashMap;

use id_arena::{Arena, Id};

#[derive(Debug, Clone)]
pub struct AstNode {
    pub kind: NodeKind,
    pub ty: Option<Id<Type>>,
    pub children: Vec<Id<AstNode>>,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Program,
    Function {
        name: String,
        params: Vec<Id<AstNode>>,
        body: Id<AstNode>,
    },
    Parameter {
        name: String,
    },
    Block,
    VariableDecl {
        name: String,
        init: Option<Id<AstNode>>,
    },
    BinaryOp {
        op: BinaryOperator,
        left: Id<AstNode>,
        right: Id<AstNode>,
    },
    Literal(Literal),
    Identifier(String),
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Int,
    Float,
    Bool,
    String,
    Function {
        params: Vec<Id<Type>>,
        ret: Id<Type>,
    },
    Unknown,
}

pub struct Compiler {
    pub ast_arena: Arena<AstNode>,
    pub type_arena: Arena<Type>,
    pub symbol_table: HashMap<String, Id<AstNode>>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            ast_arena: Arena::new(),
            type_arena: Arena::new(),
            symbol_table: HashMap::new(),
        }
    }

    pub fn build_example_ast(&mut self) -> Id<AstNode> {
        let int_type = self.type_arena.alloc(Type {
            kind: TypeKind::Int,
        });

        let x_param = self.ast_arena.alloc(AstNode {
            kind: NodeKind::Parameter {
                name: "x".to_string(),
            },
            ty: Some(int_type),
            children: vec![],
        });

        let y_param = self.ast_arena.alloc(AstNode {
            kind: NodeKind::Parameter {
                name: "y".to_string(),
            },
            ty: Some(int_type),
            children: vec![],
        });

        let x_ident = self.ast_arena.alloc(AstNode {
            kind: NodeKind::Identifier("x".to_string()),
            ty: Some(int_type),
            children: vec![],
        });

        let y_ident = self.ast_arena.alloc(AstNode {
            kind: NodeKind::Identifier("y".to_string()),
            ty: Some(int_type),
            children: vec![],
        });

        let add_expr = self.ast_arena.alloc(AstNode {
            kind: NodeKind::BinaryOp {
                op: BinaryOperator::Add,
                left: x_ident,
                right: y_ident,
            },
            ty: Some(int_type),
            children: vec![x_ident, y_ident],
        });

        let body = self.ast_arena.alloc(AstNode {
            kind: NodeKind::Block,
            ty: None,
            children: vec![add_expr],
        });

        let add_func = self.ast_arena.alloc(AstNode {
            kind: NodeKind::Function {
                name: "add".to_string(),
                params: vec![x_param, y_param],
                body,
            },
            ty: None,
            children: vec![x_param, y_param, body],
        });

        self.symbol_table.insert("add".to_string(), add_func);

        self.ast_arena.alloc(AstNode {
            kind: NodeKind::Program,
            ty: None,
            children: vec![add_func],
        })
    }

    pub fn print_ast(&self, id: Id<AstNode>, depth: usize) {
        let indent = "  ".repeat(depth);
        let node = &self.ast_arena[id];

        match &node.kind {
            NodeKind::Program => println!("{}Program", indent),
            NodeKind::Function { name, params, body } => {
                println!("{}Function: {}", indent, name);
                println!("{}  Parameters:", indent);
                for &param_id in params {
                    self.print_ast(param_id, depth + 2);
                }
                println!("{}  Body:", indent);
                self.print_ast(*body, depth + 2);
            }
            NodeKind::Parameter { name } => {
                println!(
                    "{}Parameter: {} (type: {:?})",
                    indent,
                    name,
                    node.ty.map(|t| &self.type_arena[t].kind)
                );
            }
            NodeKind::Block => {
                println!("{}Block", indent);
                for &child in &node.children {
                    self.print_ast(child, depth + 1);
                }
            }
            NodeKind::BinaryOp { op, left, right } => {
                println!("{}BinaryOp: {:?}", indent, op);
                self.print_ast(*left, depth + 1);
                self.print_ast(*right, depth + 1);
            }
            NodeKind::Identifier(name) => println!("{}Identifier: {}", indent, name),
            NodeKind::Literal(lit) => println!("{}Literal: {:?}", indent, lit),
            NodeKind::VariableDecl { name, init } => {
                println!("{}VariableDecl: {}", indent, name);
                if let Some(init_id) = init {
                    self.print_ast(*init_id, depth + 1);
                }
            }
        }
    }
}

pub struct InstructionArena {
    instructions: Arena<Instruction>,
    blocks: Arena<BasicBlock>,
}

#[derive(Debug)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
    pub result: Option<Id<Value>>,
}

#[derive(Debug)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Load,
    Store,
    Jump,
    Branch,
    Return,
}

#[derive(Debug)]
pub enum Operand {
    Value(Id<Value>),
    Block(Id<BasicBlock>),
    Immediate(i64),
}

#[derive(Debug)]
pub struct BasicBlock {
    pub label: String,
    pub instructions: Vec<Id<Instruction>>,
    pub terminator: Option<Id<Instruction>>,
}

#[derive(Debug)]
pub struct Value {
    pub name: String,
    pub ty: ValueType,
}

#[derive(Debug)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
    Ptr,
}

impl Default for InstructionArena {
    fn default() -> Self {
        Self::new()
    }
}

impl InstructionArena {
    pub fn new() -> Self {
        Self {
            instructions: Arena::new(),
            blocks: Arena::new(),
        }
    }

    pub fn create_example_ir(&mut self, values: &mut Arena<Value>) -> Id<BasicBlock> {
        let x = values.alloc(Value {
            name: "%x".to_string(),
            ty: ValueType::I32,
        });

        let y = values.alloc(Value {
            name: "%y".to_string(),
            ty: ValueType::I32,
        });

        let result = values.alloc(Value {
            name: "%result".to_string(),
            ty: ValueType::I32,
        });

        let load_x = self.instructions.alloc(Instruction {
            opcode: Opcode::Load,
            operands: vec![Operand::Value(x)],
            result: Some(x),
        });

        let load_y = self.instructions.alloc(Instruction {
            opcode: Opcode::Load,
            operands: vec![Operand::Value(y)],
            result: Some(y),
        });

        let add = self.instructions.alloc(Instruction {
            opcode: Opcode::Add,
            operands: vec![Operand::Value(x), Operand::Value(y)],
            result: Some(result),
        });

        let ret = self.instructions.alloc(Instruction {
            opcode: Opcode::Return,
            operands: vec![Operand::Value(result)],
            result: None,
        });

        self.blocks.alloc(BasicBlock {
            label: "entry".to_string(),
            instructions: vec![load_x, load_y, add],
            terminator: Some(ret),
        })
    }

    pub fn print_block(&self, block_id: Id<BasicBlock>, values: &Arena<Value>) {
        let block = &self.blocks[block_id];
        println!("{}:", block.label);

        for &inst_id in &block.instructions {
            self.print_instruction(inst_id, values);
        }

        if let Some(term_id) = block.terminator {
            self.print_instruction(term_id, values);
        }
    }

    fn print_instruction(&self, inst_id: Id<Instruction>, values: &Arena<Value>) {
        let inst = &self.instructions[inst_id];

        print!("  ");
        if let Some(result_id) = inst.result {
            print!("{} = ", values[result_id].name);
        }

        print!("{:?}", inst.opcode);

        for op in &inst.operands {
            match op {
                Operand::Value(v) => print!(" {}", values[*v].name),
                Operand::Block(b) => print!(" {}", self.blocks[*b].label),
                Operand::Immediate(i) => print!(" {}", i),
            }
        }

        println!();
    }
}

pub fn demonstrate_arena_efficiency() {
    let mut arena = Arena::<String>::new();
    let mut ids = Vec::new();

    for i in 0..1000 {
        let id = arena.alloc(format!("Node {}", i));
        ids.push(id);
    }

    println!("Arena statistics:");
    println!("  Allocated {} strings", ids.len());
    println!("  Arena len: {}", arena.len());

    let sample_ids: Vec<_> = ids.iter().step_by(100).take(5).collect();
    println!("  Sample accesses:");
    for &id in &sample_ids {
        println!("    {} -> {}", id.index(), arena[*id]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_construction() {
        let mut compiler = Compiler::new();
        let program_id = compiler.build_example_ast();

        assert_eq!(compiler.ast_arena.len(), 8);
        assert!(matches!(
            compiler.ast_arena[program_id].kind,
            NodeKind::Program
        ));
    }

    #[test]
    fn test_symbol_lookup() {
        let mut compiler = Compiler::new();
        compiler.build_example_ast();

        assert!(compiler.symbol_table.contains_key("add"));
        let func_id = compiler.symbol_table["add"];
        assert!(matches!(
            compiler.ast_arena[func_id].kind,
            NodeKind::Function { .. }
        ));
    }

    #[test]
    fn test_ir_construction() {
        let mut ir = InstructionArena::new();
        let mut values = Arena::new();
        let block_id = ir.create_example_ir(&mut values);

        assert_eq!(ir.instructions.len(), 4);
        assert_eq!(values.len(), 3);
        assert_eq!(ir.blocks[block_id].instructions.len(), 3);
    }
}
