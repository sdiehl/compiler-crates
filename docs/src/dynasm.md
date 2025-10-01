# dynasm-rs

dynasm-rs is a runtime assembler for Rust that allows you to dynamically generate and execute machine code. It provides a plugin and runtime library that work together to offer an assembly-like syntax embedded directly in Rust code. This makes it ideal for JIT compilers, runtime code specialization, and high-performance computing scenarios where static compilation isn't sufficient.

The library stands out for its compile-time syntax checking and seamless integration with Rust's type system. Unlike traditional assemblers that process text at runtime, dynasm-rs verifies assembly syntax during compilation, catching errors early while maintaining the flexibility of runtime code generation.

## Core Architecture

dynasm-rs consists of two main components: a procedural macro that processes assembly syntax at compile time, and a runtime library that manages code buffers and relocation. The macro translates assembly directives into API calls that construct machine code at runtime.

```rust
#![function!("dynasm/src/lib.rs", generate_hello_world)]
```

The `dynasm!` macro parses the assembly syntax and generates Rust code that emits the corresponding machine instructions. Labels (prefixed with `->`) are resolved automatically, handling forward and backward references transparently.

## Calling Conventions

Different platforms use different calling conventions. dynasm-rs supports multiple conventions, allowing generated code to interface correctly with external functions:

```rust
#![function!("dynasm/src/lib.rs", print)]
```

This print function uses the standard C calling convention (`extern "C"`), which on ARM64 passes the first two arguments in X0 and X1 registers. The generated assembly code follows this convention when calling the function.

## Simple Code Generation

For straightforward operations, dynasm-rs makes code generation remarkably concise:

```rust
#![function!("dynasm/src/lib.rs", generate_add_function)]
```

This generates machine code equivalent to a simple addition function. The assembly directly manipulates registers according to the calling convention, avoiding any overhead from function prologue or epilogue when unnecessary.

## Control Flow

More complex control flow patterns like recursion and loops are fully supported:

```rust
#![function!("dynasm/src/lib.rs", generate_factorial)]
```

The factorial implementation demonstrates conditional jumps, stack management for callee-saved registers, and recursive calls. The assembler handles label resolution and relative addressing automatically.

## Working with Memory

dynasm-rs excels at generating efficient memory access patterns:

```rust
#![function!("dynasm/src/lib.rs", generate_array_sum)]
```

This array summation routine showcases pointer arithmetic and loop control. The generated code is as efficient as hand-written assembly, with no abstraction overhead.

## SIMD Operations

Modern processors provide SIMD instructions for parallel data processing. dynasm-rs supports these advanced instruction sets:

```rust
#![function!("dynasm/src/lib.rs", generate_vector_add)]
```

This example demonstrates vector operations on ARM64. While the current implementation uses general-purpose registers for simplicity, ARM64's NEON instruction set provides extensive SIMD capabilities for more complex parallel operations.

## Runtime Specialization

One of dynasm-rs's key strengths is generating specialized code based on runtime information:

```rust
#![function!("dynasm/src/lib.rs", generate_multiply_by_constant)]
```

This function generates different code depending on the constant value. For powers of two, it uses efficient shift instructions instead of multiplication, demonstrating how JIT compilation can outperform static compilation for specific cases.

## Memory Management

The generated code must reside in executable memory. dynasm-rs handles the platform-specific details:

```rust
#![function!("dynasm/src/lib.rs", execute_generated_code)]
```

The library manages memory protection flags and ensures proper alignment. On Unix systems, it uses mmap with PROT_EXEC; on Windows, it uses VirtualAlloc with PAGE_EXECUTE_READWRITE.

## Label System

dynasm-rs provides a sophisticated label system for managing control flow:

- **Local labels** (prefixed with `->`) are unique within each dynasm invocation
- **Global labels** (prefixed with `=>`) can be referenced across multiple invocations
- **Dynamic labels** use runtime values for computed jumps

```rust
dynasm!(ops
    ; =>function_start:
    ; test rax, rax
    ; jz ->skip
    ; call ->helper
    ; ->skip:
    ; ret
    ; ->helper:
    ; xor rax, rax
    ; ret
);
```

## Architecture Support

dynasm-rs supports multiple architectures with comprehensive instruction set coverage:

- **ARM64/AArch64**: Modern 64-bit ARM with NEON SIMD support (demonstrated in these examples)
- **x86/x64**: Full instruction set including SSE, AVX, and AVX-512
- **ARM**: 32-bit ARM instruction sets

Each architecture has its own syntax and register naming conventions, but the overall API remains consistent. The examples in this documentation use ARM64 assembly, which is the architecture for Apple Silicon and many modern ARM processors.

## Integration Patterns

dynasm-rs integrates well with existing compiler infrastructure. Here's a pattern for compiling expressions to machine code:

```rust
enum Expr {
    Const(i32),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

fn compile_expr(expr: &Expr, ops: &mut dynasmrt::aarch64::Assembler) {
    match expr {
        Expr::Const(val) => {
            dynasm!(ops; .arch aarch64; mov w0, *val as u32);
        }
        Expr::Add(a, b) => {
            compile_expr(a, ops);
            dynasm!(ops; .arch aarch64; str w0, [sp, #-16]!);
            compile_expr(b, ops);
            dynasm!(ops; .arch aarch64; ldr w1, [sp], #16; add w0, w0, w1);
        }
        Expr::Mul(a, b) => {
            compile_expr(a, ops);
            dynasm!(ops; .arch aarch64; str w0, [sp, #-16]!);
            compile_expr(b, ops);
            dynasm!(ops; .arch aarch64; ldr w1, [sp], #16; mul w0, w0, w1);
        }
    }
}
```

This recursive compilation strategy works well for tree-structured intermediate representations.

## Performance Considerations

dynasm-rs generates code with minimal overhead:

1. **No interpretation overhead**: Generated code runs at native speed
2. **Efficient memory layout**: Code is placed contiguously in memory
3. **Smart relocation**: Labels are resolved efficiently during finalization
4. **Allocation reuse**: Assembler buffers can be reused across compilations

The primary cost is the initial code generation, making dynasm-rs ideal for code that will be executed many times.

## Best Practices

When using dynasm-rs effectively:

1. **Profile before optimizing**: Ensure dynamic generation provides real benefits
2. **Cache generated code**: Don't regenerate identical functions repeatedly
3. **Handle memory limits**: Executable memory is a finite resource
4. **Test thoroughly**: Use both unit tests and integration tests
5. **Document calling conventions**: Make ABI requirements explicit
6. **Consider alternatives**: Sometimes LLVM or Cranelift might be more appropriate

The library provides maximum flexibility, but with that comes responsibility for correctness and safety.
