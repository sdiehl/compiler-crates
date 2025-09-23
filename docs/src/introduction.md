<div align="center">
    <img src="logo.png" width="512" height="auto" alt="Compiler Crates Logo">
</div>

# Introduction

This project started as a collection of personal notes about Rust libraries useful for compiler development. As I explored different crates and built prototypes, I found myself repeatedly looking up the same patterns and examples. What began as scattered markdown files evolved into a structured internal reference document for our team.

The guide focuses on practical, compiler-specific use cases for each crate. Rather than duplicating existing documentation, it shows how these libraries solve real problems in lexing, parsing, type checking, and code generation. Each example is a working implementation that demonstrates patterns we've found effective in production compiler projects.

We're sharing this publicly in the hope that others building compilers in Rust will find it useful. The examples are intentionally concise and focused on compiler engineering tasks. All code is tested and ready to use as a starting point for your own implementations.

## Technology Stacks

Choosing the right combination of crates can significantly impact your compiler project's success. Here are our recommendations based on different use cases and experience levels.

| Use Case                 | Parsing         | Lexing | Code Generation | Error Reporting    |
| ------------------------ | --------------- | ------ | --------------- | ------------------ |
| **Simple**               | pest or chumsky | -      | cranelift       | ariadne            |
| **Rapid Prototyping**    | pest or chumsky | -      | cranelift       | codespan-reporting |
| **Performance-Critical** | lalrpop         | logos  | inkwell         | codespan-reporting |
| **Production Compilers** | lalrpop         | logos  | melior          | codespan-reporting |

The examples in this guide demonstrate these combinations in practice, showing how different crates work together to build complete compiler pipelines.
