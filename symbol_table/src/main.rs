use symbol_table_example::{
    benchmark_symbol_creation, demonstrate_compiler_context, demonstrate_concurrent_access,
    demonstrate_global_symbols, demonstrate_static_symbols, CompilerContext, Location,
    ModuleSymbolTable, SymbolInfo, SymbolKind,
};

fn main() {
    println!("=== Static Symbol Demonstration ===");
    demonstrate_static_symbols();

    println!();
    println!("=== Global Symbol Interning ===");
    demonstrate_global_symbols();

    println!();
    println!("=== Compiler Context Example ===");
    demonstrate_compiler_context();

    println!();
    println!("=== Module Symbol Table Example ===");
    let mut module = ModuleSymbolTable::new("std");

    module.define_exported(
        "println",
        SymbolInfo {
            kind: SymbolKind::Function,
            defined_at: Some(Location {
                file: symbol_table::GlobalSymbol::from("std/io.rs"),
                line: 42,
                column: 1,
            }),
            type_info: Some("fn(&str)".to_string()),
        },
    );

    module.define_exported(
        "Vec",
        SymbolInfo {
            kind: SymbolKind::Type,
            defined_at: Some(Location {
                file: symbol_table::GlobalSymbol::from("std/vec.rs"),
                line: 100,
                column: 1,
            }),
            type_info: Some("struct Vec<T>".to_string()),
        },
    );

    module.define_internal(
        "internal_helper",
        SymbolInfo {
            kind: SymbolKind::Function,
            defined_at: None,
            type_info: Some("fn() -> bool".to_string()),
        },
    );

    println!("Module '{}' symbols:", module.name.as_str());
    println!("  Exported symbols: {}", module.exported.len());
    println!("  Internal symbols: {}", module.internal.len());

    let println_sym = symbol_table::GlobalSymbol::from("println");
    if let Some(info) = module.lookup(&println_sym) {
        println!();
        println!("Lookup 'println':");
        println!("  Kind: {:?}", info.kind);
        println!("  Type: {:?}", info.type_info);
        if let Some(loc) = &info.defined_at {
            println!(
                "  Location: {}:{}:{}",
                loc.file.as_str(),
                loc.line,
                loc.column
            );
        }
    }

    println!();
    println!("=== Concurrent Symbol Cache ===");
    demonstrate_concurrent_access();

    println!();
    println!("=== Performance Comparison ===");
    benchmark_symbol_creation();

    println!();
    println!("=== Practical Lexer Example ===");
    let mut ctx = CompilerContext::new();

    let input = "if x > 5 { return x; } else { return 0; }";
    let tokens: Vec<&str> = input.split_whitespace().collect();

    println!("Input: {}", input);
    println!("Token analysis:");

    for token in tokens {
        let sym = ctx.intern_string(token);
        match ctx.is_keyword(sym) {
            Some(kind) => println!("  '{}' -> Keyword({:?})", token, kind),
            None => match token {
                "{" | "}" | ";" => println!("  '{}' -> Punctuation", token),
                ">" => println!("  '{}' -> Operator", token),
                "5" | "0" => println!("  '{}' -> Number", token),
                _ => println!("  '{}' -> Identifier", token),
            },
        }
    }

    println!();
    println!("=== Symbol Resolution Example ===");
    let source_symbols = vec![
        ("main", "function"),
        ("args", "parameter"),
        ("println", "function"),
        ("std", "module"),
        ("i", "variable"),
        ("sum", "variable"),
    ];

    println!("Building symbol resolution map...");
    let mut resolution_map = std::collections::HashMap::new();

    for (name, kind) in source_symbols {
        let sym = symbol_table::GlobalSymbol::from(name);
        resolution_map.insert(sym, kind);
        println!("  {} -> {}", name, kind);
    }

    let lookup_sym = symbol_table::GlobalSymbol::from("main");
    println!();
    println!("Looking up 'main': {:?}", resolution_map.get(&lookup_sym));
}
