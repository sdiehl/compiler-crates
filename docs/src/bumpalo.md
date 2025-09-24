# bumpalo

The `bumpalo` crate provides a fast bump allocation arena for Rust that dramatically improves allocation performance in compiler workloads. Bump allocation, also known as linear or arena allocation, allocates memory by simply incrementing a pointer through a contiguous block of memory. This makes allocation extremely fast - just a pointer bump and bounds check - at the cost of not being able to deallocate individual objects. Instead, the entire arena is deallocated at once when dropped.

For compiler development, bump allocation is ideal because compilation naturally proceeds in phases where large numbers of temporary allocations are created, used, and then all discarded together. AST nodes, type information, and intermediate representations can all be allocated in arenas that live only as long as needed. This allocation strategy eliminates the overhead of reference counting or garbage collection while providing excellent cache locality.

## Basic Allocation

The simplest use of bumpalo is allocating individual values:

```rust
#![function!("bumpalo/src/lib.rs", basic_allocation)]
```

Values allocated in the bump allocator are returned as references with the allocator's lifetime. This ensures they remain valid as long as the allocator exists.

## String Allocation

Bumpalo provides specialized methods for string allocation:

```rust
#![function!("bumpalo/src/lib.rs", allocate_strings)]
```

The `alloc_str` method is particularly efficient for building up strings during parsing or code generation, as it avoids the overhead of `String`'s capacity management.

## Slice Allocation

Copying slices into the arena is straightforward:

```rust
#![function!("bumpalo/src/lib.rs", allocate_slices)]
```

This is useful for storing parsed tokens, symbol tables, or any sequence of data that needs to outlive its original source.

## Bump-Allocated Collections

Bumpalo provides arena-allocated versions of common collections:

```rust
#![function!("bumpalo/src/lib.rs", bump_collections)]
```

These collections avoid heap allocations entirely, storing their data directly in the arena. This is perfect for temporary collections during compilation passes.

## AST Construction

Arena allocation shines for building recursive data structures like ASTs:

```rust
#![enum!("bumpalo/src/lib.rs", Expr)]
```

```rust
#![function!("bumpalo/src/lib.rs", build_ast)]
```

```rust
#![function!("bumpalo/src/lib.rs", eval_expr)]
```

The entire AST is allocated in a single contiguous memory region, providing excellent cache locality during traversal. Nodes can freely reference each other without worrying about ownership or lifetimes beyond the arena's lifetime.

## Compiler IR Structures

More complex compiler structures benefit from arena allocation:

```rust
#![struct!("bumpalo/src/lib.rs", Function)]
```

```rust
#![enum!("bumpalo/src/lib.rs", Statement)]
```

```rust
#![function!("bumpalo/src/lib.rs", build_function)]
```

This pattern works well for intermediate representations where you build up complex structures during one compilation phase and discard them after lowering or code generation.

## Reset and Reuse

Bump allocators can be reset to reclaim all memory at once:

```rust
#![function!("bumpalo/src/lib.rs", reset_and_reuse)]
```

This is perfect for compilers that process multiple files or compilation units sequentially. Reset the allocator between units to reuse the same memory.

## Scoped Allocation

Use bump allocation for temporary computations:

```rust
#![function!("bumpalo/src/lib.rs", scoped_allocation)]
```

The arena automatically frees all memory when it goes out of scope, making it ideal for temporary working memory during optimization passes.

## Higher-Order Patterns

Encapsulate arena lifetime management with closures:

```rust
#![function!("bumpalo/src/lib.rs", with_allocator)]
```

```rust
#![function!("bumpalo/src/lib.rs", closure_example)]
```

This pattern ensures the arena is properly scoped and makes it easy to add arena allocation to existing code.

## Symbol Tables

Arena allocation works well for symbol interning:

```rust
#![struct!("bumpalo/src/lib.rs", SymbolTable)]
```

```rust
#![impl!("bumpalo/src/lib.rs", SymbolTable)]
```

Interned strings live as long as the compilation unit needs them, with minimal allocation overhead and excellent cache performance.

## Graph Structures

Build complex graph structures like control flow graphs:

```rust
#![struct!("bumpalo/src/lib.rs", Node)]
```

```rust
#![function!("bumpalo/src/lib.rs", build_tree)]
```

Nodes can freely reference each other without complex lifetime management or reference counting overhead.

## Bump Boxes

For single-value allocations with ownership semantics:

```rust
#![function!("bumpalo/src/lib.rs", bump_box_example)]
```

Bump boxes provide a `Box`-like interface while using arena allocation under the hood.

## String Building

Efficient string construction without repeated allocations:

```rust
#![struct!("bumpalo/src/lib.rs", StringBuilder)]
```

```rust
#![impl!("bumpalo/src/lib.rs", StringBuilder)]
```

This avoids the repeated allocations that would occur with `String::push_str` or format strings.

## Performance Characteristics

Bump allocation provides several performance advantages for compilers:

**Allocation Speed**: O(1) allocation with just a pointer increment and bounds check. No searching for free blocks or managing free lists.

**Deallocation Speed**: O(1) for the entire arena. No need to track individual object lifetimes or run destructors.

**Memory Locality**: Sequential allocations are contiguous in memory, providing excellent cache performance during traversal.

**Low Overhead**: No per-allocation metadata like headers or reference counts. The only overhead is unused space at the end of the current chunk.

**Predictable Performance**: No garbage collection pauses or reference counting overhead. Performance is deterministic and easy to reason about.

## Best Practices

Structure your compiler passes to match arena lifetimes. Each major phase (parsing, type checking, optimization, code generation) can use its own arena that's dropped when the phase completes.

Avoid storing bump-allocated values in long-lived data structures. The arena lifetime must outlive all references to its allocated values.

Use typed arenas for hot paths. Creating type-specific arenas can eliminate pointer indirection and improve cache performance for frequently accessed types.

Reset and reuse arenas when processing multiple compilation units. This amortizes the cost of the initial memory allocation across all units.

Consider using multiple arenas for different lifetimes. For example, use one arena for the AST that lives through type checking, and another for temporary values during each optimization pass.

Profile your allocator usage to find the optimal chunk size. Larger chunks mean fewer allocations from the system allocator but potentially more wasted space.

The bumpalo crate provides a powerful tool for improving compiler performance through efficient memory management. By embracing the constraints of arena allocation - no individual deallocation - you can achieve significant performance improvements while simplifying lifetime management in your compiler implementation.
