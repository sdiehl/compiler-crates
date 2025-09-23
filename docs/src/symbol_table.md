# symbol_table

The `symbol_table` crate provides fast and concurrent symbol interning for compilers. Symbol interning is the process of storing only one copy of each distinct string value, which provides both memory efficiency and fast equality comparisons. In compiler development, symbols appear everywhere: identifiers, keywords, string literals, type names, and module paths. By interning these strings, compilers can use simple pointer comparisons instead of string comparisons, dramatically improving performance.

The crate offers two main APIs: a thread-local `SymbolTable` for single-threaded use and a global `GlobalSymbol` type that provides concurrent access across threads. The global symbols are particularly useful in modern compilers that use parallel parsing, type checking, or code generation. The `static_symbol!` macro enables compile-time symbol creation for known strings like keywords, avoiding runtime overhead entirely.

## Static Symbols

For symbols known at compile time, the `static_symbol!` macro provides the fastest possible access:

```rust
#![function!("symbol_table/src/lib.rs", demonstrate_static_symbols)]
```

Static symbols are created once and cached forever. Subsequent calls with the same string return the exact same symbol without any synchronization overhead. This makes them perfect for language keywords and built-in identifiers.

## Global Symbol Interning

The `GlobalSymbol` type provides thread-safe symbol interning:

```rust
#![function!("symbol_table/src/lib.rs", demonstrate_global_symbols)]
```

When multiple threads intern the same string, they receive symbols that compare equal and point to the same underlying string data. This enables efficient symbol sharing across parallel compiler passes.

## Compiler Context

A typical compiler pattern is to maintain a context with interned keywords and symbols:

```rust
#![struct!("symbol_table/src/lib.rs", CompilerContext)]
```

```rust
#![impl!("symbol_table/src/lib.rs", CompilerContext)]
```

This approach pre-interns all keywords during initialization, making lexical analysis faster. The `is_keyword` method becomes a simple hash lookup rather than string comparison.

## Identifiers with Spans

Compilers need to track where symbols appear in source code. Combining symbols with span information creates efficient identifier representations:

```rust
#![struct!("symbol_table/src/lib.rs", Identifier)]
```

```rust
#![impl!("symbol_table/src/lib.rs", Identifier)]
```

The identifier stores an interned symbol plus location information. The `as_str` method provides convenient access to the underlying string without allocation.

## Module Symbol Tables

Complex compilers often organize symbols by module, distinguishing between exported and internal symbols:

```rust
#![struct!("symbol_table/src/lib.rs", ModuleSymbolTable)]
```

```rust
#![impl!("symbol_table/src/lib.rs", ModuleSymbolTable)]
```

This structure efficiently represents module interfaces. Exported symbols are available to other modules, while internal symbols remain private. The lookup method searches both tables, respecting visibility rules.

## Concurrent Access Patterns

For compiler passes that need to share mutable symbol data across threads:

```rust
#![function!("symbol_table/src/lib.rs", create_concurrent_cache)]
```

```rust
#![function!("symbol_table/src/lib.rs", demonstrate_concurrent_access)]
```

This pattern uses `Arc<RwLock<HashMap>>` to allow multiple readers or a single writer. The `GlobalSymbol` keys ensure fast lookups even with concurrent access.

## Performance Characteristics

Symbol interning provides several performance benefits:

```rust
#![function!("symbol_table/src/lib.rs", benchmark_symbol_creation)]
```

The benchmark demonstrates that global symbols have competitive performance with local symbol tables while providing thread safety. The actual performance depends on contention levels and symbol reuse patterns.

## Best Practices

When using symbol_table in compiler projects, consider these guidelines:

Use `static_symbol!` for all keywords and operators known at compile time. This eliminates runtime interning overhead for the most common symbols. Create a module specifically for language keywords to centralize these definitions.

Prefer `GlobalSymbol` over local `SymbolTable` in multi-threaded compilers. The global approach simplifies code and enables better parallelization. Local tables only make sense for isolated processing with no symbol sharing.

Design data structures to store symbols rather than strings. This applies to AST nodes, type representations, and error messages. Converting to strings should only happen at boundaries like error reporting or code generation.

Be aware of symbol lifetime. Both `SymbolTable` and `GlobalSymbol` keep strings alive forever. This is rarely a problem for compilers since the set of unique identifiers is bounded, but consider the implications for long-running language servers.

Use symbols as hash map keys freely. They implement `Hash` and `Eq` with optimal performance. Many compiler algorithms become simpler when symbols can be used directly as keys.

For incremental compilation, symbols provide stable identities across compilations. Two runs that encounter the same identifier will produce symbols that compare equal, enabling effective caching strategies.