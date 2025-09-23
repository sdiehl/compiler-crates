use smallvec_example::{
    build_simple_ast, create_instruction_sequence, demonstrate_capacity, tokenize_expression,
    AstKind, AstNode, CompactError, Location, SymbolInfo, SymbolKind, SymbolTable,
};

fn main() {
    println!("=== SmallVec Capacity Demonstration ===");
    demonstrate_capacity();

    println!();
    println!("=== Tokenization Example ===");
    let expr = "x + 42 * (y - 3)";
    println!("Input: {}", expr);
    let tokens = tokenize_expression(expr);
    println!("Tokens: {} tokens generated", tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        println!("  [{}] {:?} @ {:?}", i, token.kind, token.span);
    }

    println!();
    println!("=== AST Example ===");
    let ast = build_simple_ast();
    print_ast(&ast, 0);

    println!();
    println!("=== Instruction Sequence Example ===");
    let instructions = create_instruction_sequence();
    println!("Generated {} instructions:", instructions.len());
    for (i, inst) in instructions.iter().enumerate() {
        print!("  {:04}: {:?}", i, inst.opcode);
        for op in &inst.operands {
            print!(" {:?}", op);
        }
        println!();
    }

    println!();
    println!("=== Symbol Table Example ===");
    let mut symbol_table = SymbolTable::new();

    symbol_table.insert(
        "global_var".to_string(),
        SymbolInfo {
            kind: SymbolKind::Variable,
            offset: 0,
        },
    );

    symbol_table.insert(
        "main".to_string(),
        SymbolInfo {
            kind: SymbolKind::Function,
            offset: 0x1000,
        },
    );

    symbol_table.push_scope();

    symbol_table.insert(
        "local_var".to_string(),
        SymbolInfo {
            kind: SymbolKind::Variable,
            offset: 8,
        },
    );

    symbol_table.insert(
        "param_x".to_string(),
        SymbolInfo {
            kind: SymbolKind::Parameter,
            offset: 16,
        },
    );

    println!("Symbol lookups:");
    for name in ["global_var", "main", "local_var", "param_x", "undefined"] {
        match symbol_table.lookup(name) {
            Some(info) => println!("  {}: {:?} at offset {}", name, info.kind, info.offset),
            None => println!("  {}: not found", name),
        }
    }

    println!();
    println!("After popping scope:");
    symbol_table.pop_scope();

    for name in ["global_var", "main", "local_var", "param_x"] {
        match symbol_table.lookup(name) {
            Some(info) => println!("  {}: {:?} at offset {}", name, info.kind, info.offset),
            None => println!("  {}: not found", name),
        }
    }

    println!();
    println!("=== Error Context Example ===");
    let mut error = CompactError::new(
        "Undefined variable 'foo'".to_string(),
        Location {
            file: "main.rs".to_string(),
            line: 42,
            column: 15,
        },
    );

    error.add_context(
        "In function 'calculate'".to_string(),
        Location {
            file: "main.rs".to_string(),
            line: 40,
            column: 1,
        },
    );

    println!("Error with context:");
    for (i, (msg, loc)) in error
        .messages
        .iter()
        .zip(error.locations.iter())
        .enumerate()
    {
        println!(
            "  [{}] {} at {}:{}:{}",
            i, msg, loc.file, loc.line, loc.column
        );
    }
}

fn print_ast(node: &AstNode, depth: usize) {
    let indent = "  ".repeat(depth);
    match &node.kind {
        AstKind::Program => println!("{}Program", indent),
        AstKind::Function(name) => println!("{}Function: {}", indent, name),
        AstKind::Block => println!("{}Block", indent),
        AstKind::Expression => println!("{}Expression", indent),
        AstKind::Statement => println!("{}Statement", indent),
        AstKind::Identifier(name) => println!("{}Identifier: {}", indent, name),
        AstKind::Number(value) => println!("{}Number: {}", indent, value),
    }

    for child in &node.children {
        print_ast(child, depth + 1);
    }
}
