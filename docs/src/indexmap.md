# indexmap

The `indexmap` crate provides hash maps and sets that maintain insertion order. In compiler development, preserving the order of definitions, declarations, and operations is often crucial for deterministic output, meaningful error messages, and correct code generation. While standard hash maps offer O(1) access, they randomize iteration order, which can lead to non-deterministic compiler behavior. IndexMap combines the performance of hash maps with predictable iteration order.

Compilers frequently need ordered collections for symbol tables, import lists, struct field definitions, function parameters, and type registries. IndexMap ensures that iterating over these collections always produces the same order as insertion, which is essential for reproducible builds and stable compiler output across different runs.

## Symbol Tables with Scopes

Symbol tables are fundamental to compilers, tracking identifiers and their associated information:

```rust
#![struct!("indexmap/src/lib.rs", SymbolTable)]
```

```rust
#![impl!("indexmap/src/lib.rs", SymbolTable)]
```

The symbol table uses a stack of IndexMaps to represent nested scopes. When looking up a symbol, it searches from the innermost scope outward, implementing proper lexical scoping while maintaining declaration order within each scope.

## Struct Field Layout

Struct field ordering is critical for memory layout and ABI compatibility:

```rust
#![function!("indexmap/src/lib.rs", create_struct_layout)]
```

```rust
#![function!("indexmap/src/lib.rs", demonstrate_field_ordering)]
```

IndexMap preserves field definition order while providing both name-based and index-based access. This is essential for generating correct struct layouts and for error messages that reference fields in source order.

## Import Resolution

Managing imports requires both deduplication and order preservation:

```rust
#![struct!("indexmap/src/lib.rs", ImportResolver)]
```

```rust
#![impl!("indexmap/src/lib.rs", ImportResolver)]
```

The import resolver uses IndexMap for modules and IndexSet for imported items. This ensures imports are processed in a consistent order and duplicates are automatically removed while maintaining the first occurrence position.

## Type Registry

Type systems benefit from ordered type definitions:

```rust
#![struct!("indexmap/src/lib.rs", TypeRegistry)]
```

```rust
#![impl!("indexmap/src/lib.rs", TypeRegistry)]
```

The registry maintains types in registration order, which is important for error messages, documentation generation, and ensuring primitive types are always processed before user-defined types.

## Local Variable Bindings

Tracking local variables in their declaration order helps with debugging and error reporting:

```rust
#![struct!("indexmap/src/lib.rs", LocalScope)]
```

```rust
#![impl!("indexmap/src/lib.rs", LocalScope)]
```

This generic scope structure can track any kind of bindings while preserving order. The ordered iteration is particularly useful for displaying variable dumps or generating debug information.

## Best Practices

Use IndexMap for any collection where iteration order matters for correctness or user experience. This includes symbol tables, type definitions, struct fields, function parameters, and import lists. The small overhead compared to HashMap is usually negligible compared to the benefits of deterministic behavior.

Leverage both map and index access patterns. IndexMap allows you to look up entries by key in O(1) time and also access them by position. This is useful for positional parameters, struct field offsets, and anywhere you need both named and indexed access.

Use IndexSet for ordered unique collections. Import lists, keyword sets, and type parameter bounds are good candidates. IndexSet provides the same ordering guarantees as IndexMap while ensuring uniqueness.

Consider using the `Entry` API for efficient insertions and updates. This avoids double lookups and clearly expresses the intent to either update existing entries or insert new ones.

For deterministic compilation, ensure all collections that affect output use ordered variants. This includes not just IndexMap but also considering BTreeMap for sorted output or Vec for purely sequential access.

When implementing compiler passes that transform data structures, preserve ordering information. If a pass reads from an IndexMap and produces a new collection, use IndexMap for the output to maintain order invariants throughout the compilation pipeline.

Remember that IndexMap is not a replacement for Vec when you need purely sequential access. Use Vec for instruction sequences, basic blocks, and other truly linear data. Use IndexMap when you need both key-based lookup and order preservation.
