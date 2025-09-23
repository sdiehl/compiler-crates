<div align="center">
    <img src="./docs/src/logo.png" width="512" height="auto">
</div>

# Compiler Crates

[![CI](https://github.com/sdiehl/compiler-crates/actions/workflows/ci.yml/badge.svg)](https://github.com/sdiehl/compiler-crates/actions/workflows/ci.yml)

A collection Rust crate examples focused on compiler development.

* [**Read Online**](https://sdiehl.github.io/compiler-crates/)
* [Source Code](https://github.com/sdiehl/compiler-crates)

### Parsing & Lexing

- [**bitflags**](./bitflags/src/lib.rs) - Type-safe bitmask flags for compiler IR and AST nodes
- [**chumsky**](./chumsky/src/lib.rs) - Parser combinator library with excellent error recovery
- [**lalrpop**](./lalrpop/src/lib.rs) - LR(1) parser generator with powerful grammar syntax
- [**logos**](./logos/src/lib.rs) - Fast, derive-based lexer generation
- [**nom**](./nom/src/lib.rs) - High-performance parser combinators for binary and text formats
- [**nom-locate**](./nom-locate/src/lib.rs) - Location tracking for nom parsers
- [**peg**](./peg/src/lib.rs) - Parsing expression grammar with inline syntax
- [**pest**](./pest/src/lib.rs) - PEG parser generator with elegant grammar files
- [**rowan**](./rowan/src/lib.rs) - Lossless syntax trees with incremental reparsing
- [**rust_sitter**](./rust_sitter/src/lib.rs) - Tree-sitter grammar generation through Rust macros
- [**rustc_lexer**](./rustc_lexer/src/lib.rs) - The actual lexer used by the Rust compiler

### Diagnostics

- [**codespan-reporting**](./codespan-reporting/src/lib.rs) - Beautiful compiler error messages with source snippets
- [**ariadne**](./ariadne/src/lib.rs) - Modern diagnostic reporting with emphasis on clarity
- [**miette**](./miette/src/lib.rs) - Comprehensive diagnostic framework with derive macros

### Data Structures

- [**bumpalo**](./bumpalo/src/lib.rs) - Fast bump allocation arena for compiler data structures
- [**id-arena**](./id-arena/src/lib.rs) - Efficient arena allocation for AST and IR nodes
- [**indexmap**](./indexmap/src/lib.rs) - Order-preserving hash maps for symbol tables
- [**smallvec**](./smallvec/src/lib.rs) - Stack-allocated vectors for performance-critical paths
- [**symbol_table**](./symbol_table/src/lib.rs) - String interning for compiler symbols

### Analysis & Algorithms

- [**petgraph**](./petgraph/src/lib.rs) - Graph algorithms for control flow and dependency analysis

### Code Generation

- [**cranelift**](./cranelift/src/lib.rs) - Fast JIT code generator for WebAssembly and language runtimes
- [**inkwell**](./inkwell/src/lib.rs) - Safe LLVM bindings for generating optimized machine code
- [**melior**](./melior/src/lib.rs) - MLIR bindings for multi-level IR compilation

### Parser Utilities

- [**syn**](./syn/src/lib.rs) - Rust syntax tree parsing and manipulation for procedural macros

### Development Tools

- [**codespan**](./codespan/src/lib.rs) - Core span tracking and position management for compilers
- [**rustyline**](./rustyline/src/lib.rs) - Line editing for REPL implementations

### Build Commands

```bash
just build       # Build all crates
just test        # Run all tests
just lint        # Run clippy on all crates
just format      # Format all code
just build-docs  # Build documentation
just serve-docs  # Serve documentation locally
```

## License

MIT Licensed. Copyright 2025 Stephen Diehl.
