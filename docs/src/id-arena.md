# id-arena

The `id-arena` crate provides a simple arena allocator that assigns unique IDs to values. In compiler development, arenas solve numerous challenges related to managing AST nodes, type representations, and intermediate representations. Traditional approaches using references or `Rc`/`Arc` lead to complex lifetime management and potential cycles. Arena allocation with IDs provides stable references that are copyable, comparable, and safe to store anywhere.

Compilers deal with many interconnected data structures: AST nodes reference other nodes, types reference other types, and IR instructions reference values and blocks. Using an arena with IDs instead of pointers simplifies these relationships dramatically. IDs are just integers, so they can be copied freely, stored in hash maps, and serialized without concern for lifetimes or ownership.

## AST Construction

Building an AST with id-arena involves allocating nodes in the arena and using IDs for references:

```rust
#![struct!("id-arena/src/lib.rs", AstNode)]
```

```rust
#![struct!("id-arena/src/lib.rs", Compiler)]
```

The compiler struct owns the arenas for both AST nodes and types. Nodes reference each other through IDs rather than pointers, eliminating lifetime concerns and enabling flexible tree manipulation.

## Building Complex Trees

The arena pattern makes building complex AST structures straightforward:

```rust
#![impl!("id-arena/src/lib.rs", Compiler)]
```

This example shows how function definitions, parameters, and expressions are allocated in the arena with proper parent-child relationships maintained through ID vectors.

## Intermediate Representation

Arenas work equally well for compiler IR where instructions reference values and basic blocks:

```rust
#![struct!("id-arena/src/lib.rs", InstructionArena)]
```

```rust
#![struct!("id-arena/src/lib.rs", Instruction)]
```

Instructions can reference values and blocks through IDs. The arena owns all the data, making memory management automatic and efficient.

## Type Representation

Type systems benefit from arena allocation when types can be recursive or mutually referential:

```rust
#![struct!("id-arena/src/lib.rs", Type)]
```

Function types reference parameter and return types through IDs, avoiding the complexity of boxed recursive types while maintaining type safety.

## Traversal and Printing

Arena-based structures are easy to traverse since IDs can be followed without lifetime concerns:

The print_ast method is part of the Compiler impl:

```rust
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
        // ... other node types
    }
}
```

The print function recursively follows IDs to traverse the tree. The arena provides indexed access to retrieve nodes by ID.

## Performance Benefits

Arena allocation provides several performance advantages:

```rust
#![function!("id-arena/src/lib.rs", demonstrate_arena_efficiency)]
```

Arenas allocate memory in large chunks, reducing allocator overhead. All nodes are stored contiguously, improving cache locality during traversal.

## Best Practices

When using id-arena in compiler projects, consider these guidelines:

Structure your compiler with dedicated arenas for different types of data. Separate arenas for AST nodes, types, and IR allows independent manipulation and clearer ownership. Each compiler pass can create its own arenas for intermediate data.

Use newtype wrappers around IDs when you have multiple arena types. While id-arena provides type safety through phantom types, additional newtype wrappers can prevent mixing IDs from different logical domains.

Consider arena granularity carefully. One arena for all AST nodes is simpler but prevents partial deallocation. Multiple smaller arenas allow freeing memory between compiler phases but require more careful ID management.

Leverage ID stability for caching and incremental compilation. Unlike pointers, IDs remain valid even if you add more items to the arena. This makes them ideal keys for analysis results and cached computations.

Use arena iteration for whole-program analyses. Arenas provide efficient iteration over all allocated items, useful for passes that need to examine every node, type, or instruction.

Be mindful of arena growth patterns. Arenas never shrink, so long-lived arenas in language servers or watch modes can accumulate memory. Consider periodic arena recreation for long-running processes.

Take advantage of ID copyability for parallel analysis. IDs can be freely sent between threads, enabling parallel compiler passes without complex synchronization. Each thread can safely read from shared arenas while building its own result arenas.