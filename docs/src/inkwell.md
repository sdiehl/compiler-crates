# inkwell

The `inkwell` crate provides safe, idiomatic Rust bindings to LLVM, enabling compiler developers to generate highly optimized machine code while leveraging LLVM's mature optimization infrastructure and broad platform support. LLVM IR serves as a universal intermediate representation that can be compiled to native code for virtually any modern processor architecture. The inkwell bindings wrap LLVM's C++ API with Rust's type system and ownership model, preventing common errors like use-after-free and type mismatches that plague direct LLVM usage.

The architecture of inkwell mirrors LLVM's conceptual model while providing Rust-native abstractions. Contexts manage the lifetime of LLVM data structures, modules contain functions and global variables, builders construct instruction sequences, and execution engines provide JIT compilation capabilities. This design ensures memory safety through Rust's lifetime system while maintaining the full power and flexibility of LLVM's code generation capabilities.

## Installation and Setup

Inkwell currently supports LLVM versions 8 through 18. You must have LLVM installed on your system and specify the version in your `Cargo.toml` dependencies.

On macOS, you can use Homebrew to install LLVM. For example, to install LLVM 18:

```bash
brew install llvm@18
```

Add inkwell to your `Cargo.toml` with the appropriate LLVM version feature flag:

```toml
[dependencies]
inkwell = { version = "0.6.0", features = ["llvm18-1"] }
```

Supported versions use the pattern `llvmM-0` where M is the LLVM major version (8-18).

## Basic Usage

Creating an LLVM context for code generation:

```rust
#![function!("inkwell/src/lib.rs", create_context)]
```

The context is the core of LLVM's infrastructure, owning all types, values, and metadata. Every LLVM operation requires a context, which manages memory and ensures proper cleanup of LLVM data structures.

## Function Creation

Building simple arithmetic functions:

```rust
#![function!("inkwell/src/lib.rs", create_add_function)]
```

```rust
#![function!("inkwell/src/lib.rs", create_multiply_function)]
```

Function creation involves defining the function signature through LLVM's type system, adding the function to the module, and creating basic blocks for the function body. The entry block serves as the starting point for instruction generation. Parameters are accessed through the function value and can be used in subsequent instructions.

## Constants and Comparisons

Creating constant values and comparison operations:

```rust
#![function!("inkwell/src/lib.rs", create_constant_function)]
```

```rust
#![function!("inkwell/src/lib.rs", create_comparison_function)]
```

Constants are compile-time values that LLVM can optimize aggressively. Comparison operations produce boolean results used for control flow decisions. LLVM supports both signed and unsigned integer comparisons with various predicates.

## Control Flow

Implementing conditional branches and phi nodes:

```rust
#![function!("inkwell/src/lib.rs", create_conditional_function)]
```

Control flow in LLVM uses explicit basic blocks connected by branch instructions. Conditional branches test a boolean condition and jump to one of two target blocks. Phi nodes implement the SSA form by selecting values based on the predecessor block. This explicit representation enables sophisticated control flow optimizations.

## Loop Construction

Building loops with phi nodes:

```rust
#![function!("inkwell/src/lib.rs", create_loop_function)]
```

Loops in LLVM use phi nodes to manage loop variables in SSA form. The loop structure consists of an entry block, a loop block containing the phi node and loop body, and an exit block. The phi node receives different values depending on whether control flow comes from the initial entry or from the loop itself.

## Stack Allocation

Using alloca for local variables:

```rust
#![function!("inkwell/src/lib.rs", create_alloca_function)]
```

The alloca instruction creates stack storage for mutable variables. Load and store instructions access these variables. This pattern is commonly used before [mem2reg](https://llvm.org/docs/Passes.html#mem2reg-promote-memory-to-register) optimization, which promotes allocas to SSA registers when possible.

## Array Operations

Working with arrays and pointers:

```rust
#![function!("inkwell/src/lib.rs", create_array_function)]
```

Array operations use the GEP (GetElementPtr) instruction to compute addresses of array elements. The GEP instruction performs pointer arithmetic in a type-safe manner, taking into account element sizes and array dimensions.

## Structure Types

Defining and manipulating aggregate types:

```rust
#![function!("inkwell/src/lib.rs", create_struct_function)]
```

Structures in LLVM represent aggregate data types with indexed fields. The extract_value instruction retrieves fields from struct values. This example shows how to work with heterogeneous data types in LLVM.

## Global Variables

Creating and using module-level variables:

```rust
#![function!("inkwell/src/lib.rs", create_global_variable)]
```

```rust
#![function!("inkwell/src/lib.rs", create_global_function)]
```

Global variables exist at module scope and can be accessed by all functions. They support various linkage types controlling visibility and sharing across compilation units.

## Recursive Functions

Implementing recursion:

```rust
#![function!("inkwell/src/lib.rs", create_recursive_function)]
```

Recursive functions in LLVM work like any other function call. The function can call itself by using its own function value as the callee. This example implements factorial recursively with a base case and recursive case.

## Optimization

Applying LLVM's optimization passes using the modern pass manager:

```rust
#![function!("inkwell/src/lib.rs", optimize_module)]
```

```rust
#![function!("inkwell/src/lib.rs", run_custom_passes)]
```

LLVM provides a modern pass manager (available in LLVM 18) with a string-based interface for specifying optimization pipelines. Common passes include [instcombine](https://llvm.org/docs/Passes.html#instcombine-combine-redundant-instructions), [reassociate](https://llvm.org/docs/Passes.html#reassociate-reassociate-expressions), [gvn](https://llvm.org/docs/Passes.html#gvn-global-value-numbering), [simplifycfg](https://llvm.org/docs/Passes.html#simplifycfg-simplify-the-cfg), and [mem2reg](https://llvm.org/docs/Passes.html#mem2reg-promote-memory-to-register). The PassBuilderOptions allows fine-grained control over optimization behavior.

## JIT Compilation

Just-in-time compilation for immediate execution:

```rust
#![function!("inkwell/src/lib.rs", create_execution_engine)]
```

```rust
#![function!("inkwell/src/lib.rs", jit_compile_and_execute)]
```

The execution engine provides JIT compilation capabilities, compiling LLVM IR to machine code in memory for immediate execution. This enables dynamic code generation scenarios like REPLs, runtime specialization, and adaptive optimization.

## Code Emission

Generating object files and LLVM IR:

```rust
#![function!("inkwell/src/lib.rs", compile_to_object_file)]
```

```rust
#![function!("inkwell/src/lib.rs", write_ir_to_file)]
```

```rust
#![function!("inkwell/src/lib.rs", verify_module)]
```

LLVM can emit code in various formats including object files and LLVM IR text. The target machine encapsulates platform-specific code generation details. Module verification ensures the generated IR is well-formed before optimization or code generation.

## Helper Functions

Utility for creating function types:

```rust
#![function!("inkwell/src/lib.rs", create_function_type)]
```

This helper simplifies creating function types with proper handling of void returns and variadic arguments.

## Best Practices

Maintain clear separation between your language's AST and LLVM IR generation. Build an intermediate representation that bridges your language semantics and LLVM's model. This separation simplifies both frontend development and backend optimization.

Use LLVM's type system to enforce invariants at compile time. Rich type information enables better optimization and catches errors early. Avoid using opaque pointers when specific types provide better optimization opportunities.

Leverage LLVM's SSA form by minimizing mutable state. Use phi nodes instead of memory operations when possible. SSA form enables powerful optimizations like constant propagation and dead code elimination.

Structure code generation to emit IR suitable for optimization. Avoid patterns that inhibit optimization like excessive memory operations or complex control flow. Simple, regular IR patterns optimize better than clever, complicated constructions.

Enable appropriate optimization levels based on use case. Debug builds benefit from minimal optimization for faster compilation and better debugging. Release builds should use higher optimization levels for maximum performance.

Use LLVM intrinsics for operations with hardware support. Intrinsics for mathematical functions, atomic operations, and SIMD instructions generate better code than manual implementations. LLVM recognizes and optimizes intrinsic patterns.

Profile and analyze generated code to identify optimization opportunities. LLVM provides extensive analysis passes that reveal performance bottlenecks. Use this information to guide both frontend improvements and optimization pass selection.
