# petgraph

The `petgraph` crate provides a general-purpose graph data structure library for Rust. In compiler development, graphs are fundamental data structures used to represent control flow graphs (CFG), call graphs, dependency graphs, dominance trees, and data flow analysis. The crate offers both directed and undirected graphs with flexible node and edge weights, along with a comprehensive collection of graph algorithms.

Control flow graphs are perhaps the most common use of graphs in compilers. Each node represents a basic block of instructions, and edges represent possible control flow between blocks. Call graphs track function calling relationships, which is essential for interprocedural analysis and optimization. Dependency graphs help with instruction scheduling and detecting parallelization opportunities.

## Building Control Flow Graphs

A control flow graph represents the flow of control through a program. Here we create a simple CFG for an if-then-else statement:

```rust
#![function!("petgraph/src/lib.rs", build_simple_cfg)]
```

This function builds a CFG with an entry block, a conditional block that branches to either a then or else block, and finally a merge block where control flow reconverges. The graph structure makes it easy to analyze properties like dominance relationships and reachability.

## Graph Traversal

Compilers frequently need to traverse graphs in specific orders. Depth-first search (DFS) is used for tasks like computing reverse postorder for dataflow analysis:

```rust
#![function!("petgraph/src/lib.rs", perform_dfs)]
```

Breadth-first search (BFS) is useful for level-order traversals and finding shortest paths:

```rust
#![function!("petgraph/src/lib.rs", perform_bfs)]
```

## Dominance Analysis

Dominance is a fundamental concept in compiler optimization. A node A dominates node B if every path from the entry to B must go through A:

```rust
#![function!("petgraph/src/lib.rs", find_dominators)]
```

The dominance frontier is used in SSA form construction, and immediate dominators help build the dominator tree used in many optimizations.

## Loop Detection

Detecting loops is crucial for loop optimizations. A graph contains loops if topological sorting fails:

```rust
#![function!("petgraph/src/lib.rs", build_loop_cfg)]
```

This creates a CFG with a simple while loop. The backedge from the loop body to the header creates a cycle in the graph.

## Dead Code Detection

Unreachable code detection helps identify blocks that can never be executed:

```rust
#![function!("petgraph/src/lib.rs", detect_unreachable_code)]
```

This uses path connectivity to find nodes that cannot be reached from the entry point. Such blocks can be safely removed during optimization.

## Call Graphs

Call graphs represent the calling relationships between functions in a program:

```rust
#![function!("petgraph/src/lib.rs", build_call_graph)]
```

Call graphs are essential for interprocedural analysis, inlining decisions, and detecting recursive functions:

```rust
#![function!("petgraph/src/lib.rs", find_recursive_functions)]
```

## Reverse Postorder

Reverse postorder is the standard iteration order for forward dataflow analyses:

```rust
#![function!("petgraph/src/lib.rs", reverse_postorder)]
```

This ordering ensures that when visiting a node, most of its predecessors have already been visited, leading to faster convergence in iterative dataflow algorithms.

## Graph Visualization

For debugging and understanding complex graphs, petgraph can generate DOT format output:

```rust
#![function!("petgraph/src/lib.rs", print_cfg_dot)]
```

The DOT output can be rendered with Graphviz to visualize the graph structure, which is invaluable for debugging compiler passes.

## Best Practices

Choose the appropriate graph type for your use case. Use `DiGraph` for directed graphs like CFGs and call graphs. Use `UnGraph` for undirected graphs like interference graphs in register allocation.

Node indices are not stable across node removals. If you need stable identifiers, store them in the node weight or use a separate mapping. For large graphs, consider using `StableGraph` which maintains indices across removals at a small performance cost.

Many compiler algorithms benefit from caching graph properties. For example, dominance information should be computed once and reused rather than recalculated for each query. Similarly, strongly connected components and topological orderings can be cached.

For performance-critical paths, be aware that some algorithms have different implementations with different trade-offs. The `algo` module provides both simple and optimized versions of many algorithms.

The visitor traits allow you to implement custom traversals efficiently. Use `DfsPostOrder` for postorder traversals needed in many analyses. The `visit` module provides building blocks for implementing sophisticated graph algorithms.
