# smallvec

The `smallvec` crate provides a vector type that stores a small number of elements inline, avoiding heap allocation for common cases. In compiler development, many data structures contain a small number of elements most of the time. For example, most expressions have only a few operands, most basic blocks have only a few instructions, and most functions have only a few parameters. Using `SmallVec` for these cases can significantly reduce allocations and improve cache locality.

The key insight is that compilers often deal with collections that are usually small but occasionally large. Traditional vectors always allocate on the heap, even for a single element. SmallVec stores up to N elements inline within the struct itself, only spilling to heap allocation when the capacity is exceeded. This optimization is particularly effective in AST nodes, instruction operands, and symbol tables.

## Basic Usage

SmallVec is used similarly to standard vectors but with an inline capacity specified in the type:

```rust
#![function!("smallvec/src/lib.rs", demonstrate_capacity)]
```

The type parameter `[i32; 4]` specifies both the element type and inline capacity. The vector starts with space for 4 elements allocated inline. When a fifth element is added, it spills to the heap with a larger capacity.

## Tokenization

A common use case in compilers is storing token streams. Most expressions contain a moderate number of tokens that fit well within inline storage:

```rust
#![function!("smallvec/src/lib.rs", tokenize_expression)]
```

The `TokenStream` type alias uses a SmallVec with inline capacity for 32 tokens. This covers most expressions without heap allocation while still handling arbitrarily large inputs when needed.

## AST Nodes

Abstract syntax tree nodes often have a small, variable number of children. SmallVec is ideal for storing these children:

```rust
#![struct!("smallvec/src/lib.rs", AstNode)]
```

```rust
#![enum!("smallvec/src/lib.rs", AstKind)]
```

Most AST nodes have fewer than 4 children, so this inline capacity avoids heap allocation for the common case. Function calls might have many arguments, but the vector seamlessly handles this by spilling to the heap.

## Instruction Operands

Compiler intermediate representations often model instructions with a variable number of operands:

```rust
#![struct!("smallvec/src/lib.rs", Instruction)]
```

```rust
#![function!("smallvec/src/lib.rs", create_instruction_sequence)]
```

Most instructions have 0-3 operands, making SmallVec with inline capacity of 3 an excellent choice. This keeps instruction objects compact and cache-friendly.

## Symbol Tables

Symbol tables benefit from SmallVec at multiple levels. Most scopes contain few symbols, and the scope stack itself is usually shallow:

```rust
#![struct!("smallvec/src/lib.rs", SymbolTable)]
```

```rust
#![struct!("smallvec/src/lib.rs", Scope)]
```

```rust
#![impl!("smallvec/src/lib.rs", SymbolTable)]
```

This implementation uses SmallVec for both the scope stack (usually less than 8 deep) and the symbol list within each scope (often less than 16 symbols). This provides excellent performance for typical programs while gracefully handling edge cases.

## Error Context

Compiler errors often need to track context through multiple levels. SmallVec efficiently stores this context:

```rust
#![struct!("smallvec/src/lib.rs", CompactError)]
```

Most errors have only one or two context levels, so inline storage of 2 elements covers the common case without allocation.

## Best Practices

Choose inline capacity based on profiling and typical use cases. Too small wastes the optimization opportunity, while too large wastes stack space. Common sweet spots are 2-4 for AST children, 8-16 for local collections, and 32-64 for token buffers.

Be aware of the size implications. A `SmallVec<[T; N]>` is approximately the size of N elements plus a discriminant and pointer. This can make structs larger, potentially affecting cache behavior. Measure the trade-offs in your specific use case.

Use type aliases to make code more readable and to centralize capacity decisions. This makes it easy to tune capacities based on profiling data.

Consider using SmallVec in hot paths where allocation overhead matters. Parser combinators, visitor patterns, and iterative algorithms often benefit significantly.

The `smallvec!` macro provides convenient initialization similar to `vec!`. Use it for clarity when creating SmallVecs with initial values.

For recursive structures like ASTs, SmallVec can dramatically reduce total allocations. A tree with depth D and branching factor B would normally require O(B^D) allocations, but with SmallVec, most nodes require zero heap allocations.
