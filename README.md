<div align="center">
    <img src="./docs/src/logo.png" width="512" height="auto">
</div>

# Compiler Crates

[![CI](https://github.com/sdiehl/compiler-crates/actions/workflows/ci.yml/badge.svg)](https://github.com/sdiehl/compiler-crates/actions/workflows/ci.yml)

A collection of minimal Rust examples focused on compiler development.

* [**Read Online**](https://sdiehl.github.io/compiler-crates/)
* [Source Code](https://github.com/sdiehl/compiler-crates)

### Parsing & Lexing

- [**chumsky**](./chumsky/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/chumsky.html) - Parser combinator library with excellent error recovery
- [**combine**](./combine/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/combine.html) - Parser combinators with zero-copy mode for performance
- [**lalrpop**](./lalrpop/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/lalrpop.html) - LR(1) parser generator with powerful grammar syntax
- [**logos**](./logos/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/logos.html) - Fast, derive-based lexer generation
- [**nom**](./nom/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/nom.html) - High-performance parser combinators for binary and text formats
- [**peg**](./peg/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/peg.html) - Parsing expression grammar with inline syntax
- [**pest**](./pest/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/pest.html) - PEG parser generator with elegant grammar files
- [**rowan**](./rowan/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/rowan.html) - Lossless syntax trees with incremental reparsing
- [**rust_sitter**](./rust_sitter/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/rust_sitter.html) - Tree-sitter grammar generation through Rust macros
- [**rustc_lexer**](./rustc_lexer/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/rustc_lexer.html) - The actual lexer used by the Rust compiler
- [**winnow**](./winnow/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/winnow.html) - Parser combinators with a focus on maintainability

### Parser Utilities

- [**nom_locate**](./nom_locate/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/nom_locate.html) - Location tracking for nom parsers
- [**quote**](./quote/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/quote.html) - Quasi-quoting for Rust code generation
- [**syn**](./syn/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/syn.html) - Rust syntax tree parsing and manipulation for procedural macros

### Diagnostics

- [**ariadne**](./ariadne/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/ariadne.html) - Modern diagnostic reporting with emphasis on clarity
- [**codespan-reporting**](./codespan-reporting/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/codespan-reporting.html) - Beautiful compiler error messages with source snippets
- [**miette**](./miette/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/miette.html) - Comprehensive diagnostic framework with derive macros

### Data Structures

- [**bitflags**](./bitflags/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/bitflags.html) - Type-safe bitmask flags for compiler IR and AST nodes
- [**bumpalo**](./bumpalo/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/bumpalo.html) - Fast bump allocation arena for compiler data structures
- [**id-arena**](./id-arena/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/id-arena.html) - Efficient arena allocation for AST and IR nodes
- [**indexmap**](./indexmap/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/indexmap.html) - Order-preserving hash maps for symbol tables
- [**smallvec**](./smallvec/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/smallvec.html) - Stack-allocated vectors for performance-critical paths
- [**symbol_table**](./symbol_table/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/symbol_table.html) - String interning for compiler symbols

### Analysis & Algorithms

- [**petgraph**](./petgraph/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/petgraph.html) - Graph algorithms for control flow and dependency analysis

### Code Generation

- [**cranelift**](./cranelift/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/cranelift.html) - Fast JIT code generator for WebAssembly and language runtimes
- [**dynasm**](./dynasm/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/dynasm.html) - Runtime assembler for ARM64 and x86-64 with compile-time syntax checking
- [**inkwell**](./inkwell/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/inkwell.html) - Safe LLVM bindings for generating optimized machine code
- [**melior**](./melior/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/melior.html) - MLIR bindings for multi-level IR compilation

### Development Tools

- [**codespan**](./codespan/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/codespan.html) - Core span tracking and position management for compilers
- [**rustyline**](./rustyline/src/lib.rs) | [docs](https://sdiehl.github.io/compiler-crates/rustyline.html) - Line editing for REPL implementations

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
