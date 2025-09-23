use indexmap::{IndexMap, IndexSet};
use indexmap_example::{
    demonstrate_field_ordering, demonstrate_import_resolution, LocalScope, Symbol, SymbolKind,
    SymbolTable, TypeDefinition, TypeKind, TypeRegistry,
};

fn main() {
    println!("=== IndexMap for Struct Field Ordering ===");
    demonstrate_field_ordering();

    println!();
    println!("=== Symbol Table with Scopes ===");
    let mut symbol_table = SymbolTable::new();

    symbol_table.insert(
        "PI".to_string(),
        Symbol {
            name: "PI".to_string(),
            kind: SymbolKind::Variable {
                mutable: false,
                ty: "f64".to_string(),
            },
            scope_level: 0,
        },
    );

    symbol_table.insert(
        "main".to_string(),
        Symbol {
            name: "main".to_string(),
            kind: SymbolKind::Function {
                params: vec![],
                ret_ty: "()".to_string(),
            },
            scope_level: 0,
        },
    );

    println!("Global scope symbols:");
    for (name, symbol) in symbol_table.current_scope_symbols() {
        println!("  {} -> {:?}", name, symbol.kind);
    }

    symbol_table.push_scope();

    symbol_table.insert(
        "x".to_string(),
        Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable {
                mutable: true,
                ty: "i32".to_string(),
            },
            scope_level: 1,
        },
    );

    symbol_table.insert(
        "y".to_string(),
        Symbol {
            name: "y".to_string(),
            kind: SymbolKind::Variable {
                mutable: false,
                ty: "String".to_string(),
            },
            scope_level: 1,
        },
    );

    println!();
    println!("Inner scope symbols (in insertion order):");
    for (name, symbol) in symbol_table.current_scope_symbols() {
        println!("  {} -> {:?}", name, symbol.kind);
    }

    println!();
    println!("Symbol lookup results:");
    println!("  'x' found: {}", symbol_table.lookup("x").is_some());
    println!("  'PI' found: {}", symbol_table.lookup("PI").is_some());
    println!("  'z' found: {}", symbol_table.lookup("z").is_some());

    println!();
    println!("=== Import Resolution ===");
    demonstrate_import_resolution();

    println!();
    println!("=== Type Registry ===");
    let mut type_registry = TypeRegistry::new();

    type_registry.register_type(TypeDefinition {
        name: "Token".to_string(),
        kind: TypeKind::Enum {
            variants: IndexSet::from([
                "Identifier".to_string(),
                "Number".to_string(),
                "Operator".to_string(),
                "Keyword".to_string(),
            ]),
        },
    });

    type_registry.register_type(TypeDefinition {
        name: "AstNode".to_string(),
        kind: TypeKind::Struct {
            fields: IndexMap::from([
                ("kind".to_string(), "NodeKind".to_string()),
                ("span".to_string(), "Span".to_string()),
                ("children".to_string(), "Vec<AstNode>".to_string()),
            ]),
        },
    });

    println!("Registered types (in registration order):");
    for (name, def) in type_registry.iter_types() {
        match &def.kind {
            TypeKind::Primitive => println!("  {} (primitive)", name),
            TypeKind::Struct { fields } => {
                println!("  {} (struct with {} fields)", name, fields.len())
            }
            TypeKind::Enum { variants } => {
                println!("  {} (enum with {} variants)", name, variants.len())
            }
            TypeKind::Alias { target } => println!("  {} (alias for {})", name, target),
        }
    }

    println!();
    println!("=== Local Variable Bindings ===");
    let mut scope: LocalScope<String, i32> = LocalScope::new();

    scope.bind("x".to_string(), 10);
    scope.bind("y".to_string(), 20);
    scope.bind("z".to_string(), 30);
    scope.bind("x".to_string(), 15);

    println!("Variables in binding order:");
    for (name, value) in scope.bindings_ordered() {
        println!("  {} = {}", name, value);
    }
}
