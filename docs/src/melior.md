# melior

The `melior` crate provides safe Rust bindings to MLIR (Multi-Level Intermediate Representation), enabling compiler developers to leverage MLIR's powerful infrastructure for building optimizing compilers and domain-specific code generators. MLIR represents computations as a graph of operations organized into regions and blocks, supporting multiple levels of abstraction from high-level tensor operations to low-level machine instructions. The melior bindings expose MLIR's dialect system, allowing developers to work with various IR representations including arithmetic operations, control flow, tensor computations, and LLVM IR generation.

The architecture of melior wraps MLIR's C API with idiomatic Rust abstractions, providing type safety and memory safety guarantees while maintaining the flexibility of MLIR's extensible design. The crate supports all standard MLIR dialects including [func](https://mlir.llvm.org/docs/Dialects/Func/), [arith](https://mlir.llvm.org/docs/Dialects/ArithOps/), [scf](https://mlir.llvm.org/docs/Dialects/SCFDialect/), [tensor](https://mlir.llvm.org/docs/Dialects/TensorOps/), [memref](https://mlir.llvm.org/docs/Dialects/MemRef/), and [llvm](https://mlir.llvm.org/docs/Dialects/LLVM/), enabling progressive lowering from high-level abstractions to executable code. This multi-level approach allows compilers to perform optimizations at the most appropriate abstraction level, improving both compilation time and generated code quality.

## Installation on macOS

To use melior on macOS, you need to install LLVM/MLIR 20 via Homebrew:

```bash
brew install llvm@20
```

You can get the LLVM installation path with:

```bash
$(brew --prefix llvm@20)
```

Add melior to your Cargo.toml:

```toml
[dependencies]
melior = "0.25"
```

## Basic Usage

Creating an MLIR context with all dialects loaded:

```rust
#![function!("melior/src/lib.rs", create_test_context)]
```

The context manages dialect registration and configuration. MLIR requires dialects to be loaded before you can use their operations. The `create_test_context` function loads all standard dialects and LLVM translations for immediate use.

## Function Creation

Building simple arithmetic functions:

```rust
#![function!("melior/src/lib.rs", create_add_function)]
```

```rust
#![function!("melior/src/lib.rs", create_multiply_function)]
```

Function creation involves specifying parameter types and return types using MLIR's type system. The function body consists of a region containing basic blocks, with the entry block receiving function parameters as block arguments. This structure supports both simple functions and complex control flow patterns.

## Arithmetic Operations

Creating constant values and arithmetic computations:

```rust
#![function!("melior/src/lib.rs", create_constant)]
```

The [arith dialect](https://mlir.llvm.org/docs/Dialects/ArithOps/) provides integer and floating-point arithmetic operations. Constants are materialized using arith::constant operations, and computations build expression trees through operation chaining. Each operation produces results that subsequent operations consume, creating a dataflow graph representation.

## Type and Attribute Builders

Helper utilities for creating MLIR types and attributes:

```rust
#![struct!("melior/src/lib.rs", TypeBuilder)]
```

```rust
#![impl!("melior/src/lib.rs", TypeBuilder)]
```

```rust
#![struct!("melior/src/lib.rs", AttributeBuilder)]
```

```rust
#![impl!("melior/src/lib.rs", AttributeBuilder)]
```

These builders provide convenient methods for creating common MLIR types and attributes without manually constructing them each time. The TypeBuilder handles integer types, index types, and function types. The AttributeBuilder creates string attributes, integer attributes, and type attributes.

## Module Operations

Utility functions for working with MLIR modules:

```rust
#![function!("melior/src/lib.rs", verify_module)]
```

```rust
#![function!("melior/src/lib.rs", module_to_string)]
```

These utilities help verify module correctness and convert modules to their textual MLIR representation for debugging and inspection.

## MLIR Transformations and Optimization Passes

MLIR's power comes from its transformation infrastructure. The PassManager orchestrates optimization passes that transform and optimize IR at different abstraction levels.

### Basic Transformations

[Canonicalizer](https://mlir.llvm.org/docs/Passes/#-canonicalize) simplifies IR by applying local pattern-based rewrites:

```rust
#![function!("melior/src/lib.rs", apply_canonicalization)]
```

[Common Subexpression Elimination (CSE)](https://mlir.llvm.org/docs/Passes/#-cse) removes redundant computations:

```rust
#![function!("melior/src/lib.rs", apply_cse)]
```

[Sparse Conditional Constant Propagation (SCCP)](https://mlir.llvm.org/docs/Passes/#-sccp) performs constant folding and dead code elimination:

```rust
#![function!("melior/src/lib.rs", apply_sccp)]
```

### Function Optimizations

[Inlining](https://mlir.llvm.org/docs/Passes/#-inline) replaces function calls with their bodies:

```rust
#![function!("melior/src/lib.rs", apply_inlining)]
```

[Symbol DCE](https://mlir.llvm.org/docs/Passes/#-symbol-dce) removes unused functions and global symbols:

```rust
#![function!("melior/src/lib.rs", apply_symbol_dce)]
```

### Loop Optimizations

[Loop-Invariant Code Motion (LICM)](https://mlir.llvm.org/docs/Passes/#-loop-invariant-code-motion) hoists invariant computations out of loops:

```rust
#![function!("melior/src/lib.rs", apply_licm)]
```

### Memory Optimizations

Promote memory allocations to SSA registers using [mem2reg](https://mlir.llvm.org/docs/Passes/#-mem2reg):

```rust
#![function!("melior/src/lib.rs", apply_mem2reg)]
```

### GPU Transformations

Convert parallel patterns to GPU kernels using [GPU dialect lowering](https://mlir.llvm.org/docs/Dialects/GPU/):

```rust
#![function!("melior/src/lib.rs", convert_to_gpu)]
```

### Utility Passes

Strip debug information for release builds using [strip-debuginfo](https://mlir.llvm.org/docs/Passes/#-strip-debuginfo):

```rust
#![function!("melior/src/lib.rs", strip_debug_info)]
```

### Optimization Pipelines

Combine multiple passes into an optimization pipeline:

```rust
#![function!("melior/src/lib.rs", optimize_module)]
```

### Custom Pass Pipelines

Build fluent transformation pipelines with the PassPipeline builder:

```rust
#![struct!("melior/src/lib.rs", PassPipeline)]
```

```rust
#![impl!("melior/src/lib.rs", PassPipeline)]
```

The PassPipeline builder provides a fluent API for constructing custom optimization sequences. Each transformation method returns self, allowing method chaining. The pipeline executes passes in the order they were added, enabling precise control over optimization phases.

## Best Practices

Structure compilation as progressive lowering through multiple abstraction levels. Start with domain-specific representations and lower gradually to executable code. This approach enables optimizations at appropriate abstraction levels and improves compiler modularity.

Leverage MLIR's dialect system to separate concerns. Use high-level dialects for domain logic, mid-level dialects for general optimizations, and low-level dialects for code generation. This separation enables reuse across different compilation pipelines.

Design custom operations to be composable and orthogonal. Avoid monolithic operations that combine multiple concepts. Instead, build complex behaviors from simple, well-defined operations that optimization passes can analyze and transform.

Use MLIR's type system to enforce invariants. Rich types catch errors early and enable optimizations. Track properties like tensor dimensions, memory layouts, and value constraints through the type system rather than runtime checks.

Implement verification for custom operations. Verification catches IR inconsistencies early and provides better error messages. Well-verified IR enables aggressive optimizations without compromising correctness.

Build reusable transformation patterns. Patterns that match common idioms enable optimization across different contexts. Parameterize patterns to handle variations while maintaining transformation correctness.

The melior bindings make MLIR's powerful infrastructure accessible to Rust developers, enabling construction of sophisticated optimizing compilers with modern tooling and safety guarantees. The combination of multiple abstraction levels, extensible dialects, and powerful optimization infrastructure provides a solid foundation for next-generation compiler development.
