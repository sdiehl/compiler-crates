use id_arena::Arena;
use id_arena_example::{
    demonstrate_arena_efficiency, AstNode, BinaryOperator, Compiler, InstructionArena, Literal,
    NodeKind, Type, TypeKind,
};

fn main() {
    println!("=== AST Construction with id-arena ===");
    let mut compiler = Compiler::new();
    let program_id = compiler.build_example_ast();

    println!("AST Arena Statistics:");
    println!("  Total nodes: {}", compiler.ast_arena.len());
    println!("  Total types: {}", compiler.type_arena.len());
    println!("  Symbol table entries: {}", compiler.symbol_table.len());

    println!();
    println!("AST Structure:");
    compiler.print_ast(program_id, 0);

    println!();
    println!("=== IR Construction Example ===");
    let mut ir = InstructionArena::new();
    let mut values = Arena::new();
    let entry_block = ir.create_example_ir(&mut values);

    println!("IR for function 'add':");
    ir.print_block(entry_block, &values);

    println!();
    println!("=== Arena Efficiency Demonstration ===");
    demonstrate_arena_efficiency();

    println!();
    println!("=== Complex AST Example ===");
    let mut complex_compiler = Compiler::new();

    let int_type = complex_compiler.type_arena.alloc(Type {
        kind: TypeKind::Int,
    });
    let bool_type = complex_compiler.type_arena.alloc(Type {
        kind: TypeKind::Bool,
    });

    let x_init = complex_compiler.ast_arena.alloc(AstNode {
        kind: NodeKind::Literal(Literal::Integer(10)),
        ty: Some(int_type),
        children: vec![],
    });

    let x_var = complex_compiler.ast_arena.alloc(AstNode {
        kind: NodeKind::VariableDecl {
            name: "x".to_string(),
            init: Some(x_init),
        },
        ty: None,
        children: vec![],
    });

    let y_init = complex_compiler.ast_arena.alloc(AstNode {
        kind: NodeKind::Literal(Literal::Integer(20)),
        ty: Some(int_type),
        children: vec![],
    });

    let y_var = complex_compiler.ast_arena.alloc(AstNode {
        kind: NodeKind::VariableDecl {
            name: "y".to_string(),
            init: Some(y_init),
        },
        ty: None,
        children: vec![],
    });

    let x_ref = complex_compiler.ast_arena.alloc(AstNode {
        kind: NodeKind::Identifier("x".to_string()),
        ty: Some(int_type),
        children: vec![],
    });

    let y_ref = complex_compiler.ast_arena.alloc(AstNode {
        kind: NodeKind::Identifier("y".to_string()),
        ty: Some(int_type),
        children: vec![],
    });

    let comparison = complex_compiler.ast_arena.alloc(AstNode {
        kind: NodeKind::BinaryOp {
            op: BinaryOperator::Lt,
            left: x_ref,
            right: y_ref,
        },
        ty: Some(bool_type),
        children: vec![x_ref, y_ref],
    });

    let block = complex_compiler.ast_arena.alloc(AstNode {
        kind: NodeKind::Block,
        ty: None,
        children: vec![x_var, y_var, comparison],
    });

    println!(
        "Complex AST with {} nodes:",
        complex_compiler.ast_arena.len()
    );
    complex_compiler.print_ast(block, 0);

    println!();
    println!("=== Memory Efficiency ===");
    println!(
        "Size of Id<AstNode>: {} bytes",
        std::mem::size_of::<id_arena::Id<AstNode>>()
    );
    println!(
        "Size of Box<AstNode>: {} bytes",
        std::mem::size_of::<Box<AstNode>>()
    );
    println!(
        "Size of &AstNode: {} bytes",
        std::mem::size_of::<&AstNode>()
    );

    println!();
    println!("Benefits of id-arena:");
    println!("  - Stable IDs that survive mutations");
    println!("  - No reference lifetime issues");
    println!("  - Cache-friendly iteration");
    println!("  - Easy serialization/deserialization");
    println!("  - Efficient memory usage for trees");
}
