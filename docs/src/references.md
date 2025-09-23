# References

## Foundational Compiler Textbooks

* [Compilers: Principles, Techniques, and Tools (The Dragon Book)](https://www.pearson.com/en-us/subject-catalog/p/compilers-principles-techniques-and-tools/P200000003159) - The definitive classic textbook by Aho, Lam, Sethi, and Ullman. Comprehensive treatment of lexical analysis, parsing, semantic analysis, and code generation with formal foundations. Updated in 2006 with modern optimization techniques and garbage collection coverage.

* [Engineering a Compiler (3rd Edition)](https://www.elsevier.com/books/engineering-a-compiler/cooper/978-0-12-815412-0) - Keith Cooper and Linda Torczon's practical engineering approach to compiler construction. Covers SSA forms, instruction scheduling, and graph-coloring register allocation.

* [Modern Compiler Implementation in C/Java/ML](https://www.cs.princeton.edu/~appel/modern/) - Andrew Appel's series providing detailed coverage of all compiler phases with working implementations. Available in three language editions. Excellent treatment of advanced topics including object-oriented and functional language compilation.

* [Introduction to Compilers and Language Design](https://dthain.github.io/books/compiler/) - Douglas Thain's modern, accessible textbook offering a one-semester introduction. Enables building a simple compiler for a C-like language targeting X86 or ARM assembly with complete code examples.

## Practical Hands-On Guides

* [Crafting Interpreters](https://craftinginterpreters.com/) - Robert Nystrom's exceptional hands-on guide building two complete interpreters from scratch. Free online. Covers parsing, semantic analysis, garbage collection, and optimization with beautiful hand-drawn illustrations.

* [Building an Optimizing Compiler](https://www.amazon.com/Building-Optimizing-Compiler-Robert-Morgan/dp/155558179X) - Robert Morgan's advanced treatment of optimization techniques including data flow analysis, SSA form, and advanced optimization passes.

* [Writing a C Compiler: Build a Real Programming Language from Scratch](https://nostarch.com/writing-c-compiler) - Nora Sandler's book providing a clear path through compiler construction complexities. Progressive approach from simple programs to advanced features using pseudocode for any-language implementation.

* [Essentials of Compilation: An Incremental Approach in Python](https://github.com/IUCompilerCourse/Essentials-of-Compilation) - Jeremy Siek's unique incremental approach building a compiler progressively. Makes abstract concepts tangible through direct Python implementation, connecting language design decisions with compiler implementation.

* [Let's Build a Compiler](https://compilers.iecc.com/crenshaw/) - Jack Crenshaw's practically-oriented tutorial demystifying compiler internals. Step-by-step approach presenting up-to-date techniques with detailed implementation guidance.

## Specialized Topics

* [Advanced Compiler Design and Implementation](https://www.amazon.com/Advanced-Compiler-Design-Implementation-Muchnick/dp/1558603204) - Steven Muchnick's comprehensive treatment of advanced compiler optimization techniques. Covers case studies of commercial compilers from Sun, IBM, DEC, and Intel. Introduces Informal Compiler Algorithm Notation (ICAN) for clear algorithm communication.

* [Parsing Techniques: A Practical Guide (2nd Edition)](https://dickgrune.com/Books/PTAPG_2nd_Edition/) - Dick Grune and Ceriel Jacobs' definitive 622-page treatment of parsing techniques. Free PDF available. Covers all parsing methods with clear explanations and practical applicability.

* [Types and Programming Languages](https://www.cis.upenn.edu/~bcpierce/tapl/) - Benjamin Pierce's definitive reference for understanding type systems. While not specifically a compiler book, it's crucial for semantic analysis in compilers. Covers type checking, type inference, and advanced type system features.

* [Garbage Collection: Algorithms for Automatic Dynamic Memory Management](https://www.cs.kent.ac.uk/people/staff/rej/gcbook/) - Richard Jones and Rafael Lins' comprehensive survey of garbage collection algorithms. Covers all major collection strategies including mark-sweep, copying, generational, and concurrent collectors.

## LLVM Infrastructure

* [Learn LLVM 17: A Beginner's Guide](https://www.packtpub.com/product/learn-llvm-17/9781837631346) - Kai Nacke and Amy Kwan's hands-on guide to building and extending LLVM compilers. Covers frontend construction, backend development, IR generation and optimization, custom passes, and JIT compilation.

* [LLVM Tutorial: Kaleidoscope](https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/index.html) - Official step-by-step tutorial building a simple language frontend with LLVM. Covers lexing, parsing, AST construction, LLVM IR generation, JIT compilation, and optimization.

* [Clang Compiler Frontend](https://www.packtpub.com/product/clang-compiler-frontend/9781837630981) - Ivan Murashko's exploration of Clang internals with practical applications for static analysis and custom tooling. Covers AST operations, IDE integration, and performance optimization.

* [LLVM's Analysis and Transform Passes](https://llvm.org/docs/Passes.html) - Documentation of LLVM's optimization passes, useful for understanding what optimizations production compilers implement.

## MLIR Infrastructure

* [MLIR Passes](https://mlir.llvm.org/docs/Passes/) - Comprehensive documentation of MLIR's transformation and analysis passes. Covers affine loop transformations, buffer optimizations, control flow simplifications, and dialect-specific passes for GPU, async, linalg, and other domains.

* [MLIR Tutorial](https://mlir.llvm.org/docs/Tutorials/Toy/) - Step-by-step guide building a compiler for the Toy language using MLIR. Demonstrates how to define dialects, implement lowering passes, and leverage MLIR's infrastructure for optimization.

* [MLIR Dialect Conversion](https://mlir.llvm.org/docs/DialectConversion/) - Guide to MLIR's dialect conversion framework for progressive lowering between abstraction levels. Essential for understanding how to transform between different IR representations.

* [MLIR Pattern Rewriting](https://mlir.llvm.org/docs/PatternRewriter/) - Documentation on MLIR's declarative pattern rewriting infrastructure. Shows how to express transformations as patterns for maintainable optimization passes.

## Cranelift Resources

* [Cranelift's Instruction Selector DSL, ISLE: Term-Rewriting Made Practical](https://cfallin.org/blog/2023/01/20/cranelift-isle/) - Deep dive into Cranelift's instruction selection system using a custom term-rewriting DSL. Shows how to map IR operations to machine instructions systematically.

* [Cranelift, Part 4: A New Register Allocator](https://cfallin.org/blog/2022/06/09/cranelift-regalloc2/) - Detailed exploration of Cranelift's register allocation algorithm, covering live ranges, interference graphs, and the practical engineering of a production register allocator.

* [Cranelift: Using E-Graphs for Verified, Cooperating Middle-End Optimizations](https://github.com/bytecodealliance/rfcs/blob/main/accepted/cranelift-egraph.md) - RFC describing how Cranelift uses e-graphs to solve the phase-ordering problem in compiler optimizations while maintaining correctness guarantees.

## Language-Specific Implementation

* [Compiling to Assembly from Scratch](https://keleshev.com/compiling-to-assembly-from-scratch/) - Vladimir Keleshev's modern approach using TypeScript subset targeting ARM assembly. Covers both baseline compiler and advanced extensions with complete source code.

* [Implementing Functional Languages: A Tutorial](https://www.microsoft.com/en-us/research/publication/implementing-functional-languages-a-tutorial/) - Simon Peyton Jones and David Lester's guide to implementing non-strict functional languages. Free PDF. Includes complete working prototypes using lazy graph reduction.

* [Write You a Haskell](http://dev.stephendiehl.com/fun/) - My old tutorial on functional language implementation including parser, type inference, pattern matching, typeclasses, STG intermediate language, and native code generation.

* [Compiler Construction](https://people.inf.ethz.ch/wirth/CompilerConstruction/) - Niklaus Wirth's concise, practical guide. Step-by-step approach through each compiler design stage focusing on practical implementation.

## Parsing Tools and Techniques

* [The Definitive ANTLR 4 Reference](https://pragprog.com/titles/tpantlr2/the-definitive-antlr-4-reference/) - Terence Parr's essential guide to ANTLR parser generator with LL(*) parsing technology. Covers grammar construction, tree construction, and StringTemplate code generation.

* [ANTLR Mega Tutorial](https://tomassetti.me/antlr-mega-tutorial/) - Federico Tomassetti's comprehensive tutorial covering ANTLR setup for multiple languages (JavaScript, Python, Java, C#), testing approaches, and advanced features.

* [LR Parsing Theory and Practice](https://blog.reverberate.org/2013/07/ll-and-lr-parsing-demystified.html) - Excellent blog post demystifying the differences between LL and LR parsing with practical examples.

## Courses

* [CS 6120: Advanced Compilers: The Self-Guided Online Course](https://www.cs.cornell.edu/courses/cs6120/2020fa/self-guided/) - Cornell's graduate-level compiler optimization course. Covers SSA form, dataflow analysis, loop optimizations, and modern optimization techniques with hands-on projects.

* [Stanford CS 143: Compilers](https://web.stanford.edu/class/cs143/) - Introduction to compiler construction covering lexical analysis through code generation. Includes programming assignments building a compiler for a Java-like language.

* [IU P423/P523 Compilers](https://github.com/IUCompilerCourse) - Jeremy Siek's course using incremental approach with Racket. Materials available on GitHub.

* [KAIST CS420 Compiler Design](https://github.com/kaist-cp/cs420) - Modern treatment with Rust implementation. Course materials and assignments available on GitHub.

## Online Resources and Tutorials

* [Basics of Compiler Design](http://www.diku.dk/~torbenm/Basics/basics_lulu2.pdf) - Torben Mogensen's free PDF textbook providing solid introduction to compiler construction fundamentals.

* [Compiler Design Tutorials](https://www.geeksforgeeks.org/compiler-design-tutorials/) - Collection of articles covering compiler topics from basic to advanced with code examples.

## Research Papers and Academic Resources

* [Static Single Assignment Form and the Control Dependence Graph](https://www.cs.utexas.edu/~pingali/CS380C/2010/papers/ssaCytron.pdf) - Cytron et al.'s seminal paper on SSA form, now standard in modern optimizing compilers.

* [A Nanopass Framework for Compiler Education](https://www.cs.indiana.edu/~dyb/pubs/nano-jfp.pdf) - Describes breaking compiler passes into tiny transformations, making complex optimizations easier to understand and verify.

* [Linear Scan Register Allocation](http://www.christianwimmer.at/Publications/Wimmer04a/Wimmer04a.pdf) - Massimiliano Poletto and Vivek Sarkar's influential paper on fast register allocation suitable for JIT compilers.

## Rust-Specific Compiler Resources

* [Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/) - The official guide to rustc internals. Essential reading for understanding how a production Rust compiler works.

* [Salsa](https://salsa-rs.github.io/salsa/) - Framework for incremental computation used by rust-analyzer. Demonstrates modern techniques for responsive compiler frontends.

* [Make A Language](https://arzg.github.io/lang/) - Series of blog posts walking through implementing a programming language in Rust, from lexing through type checking.

* [Introduction to LLVM in Rust](https://github.com/jauhien/iron-kaleidoscope) - Rust implementation of the LLVM Kaleidoscope tutorial demonstrating LLVM bindings.

## Code Generation Resources

* [Cranelift Code Generator](https://github.com/bytecodealliance/wasmtime/tree/main/cranelift) - Production code generator written in Rust, designed for JIT and AOT compilation. Good example of modern compiler backend architecture.

* [Introduction to LLVM](https://mukulrathi.com/create-your-own-programming-language/llvm-ir-cpp-api-tutorial/) - Tutorial on using LLVM's C++ API to generate code, covering the basics of LLVM IR and the programmatic interface.

* [Compiler Design in C](https://holub.com/goodies/compiler/compilerDesignInC.pdf) - Allen Holub's 924-page detailed coverage of real-world compiler implementation focusing on code generation.


## Tools and Development Environments

* [Tree-sitter](https://tree-sitter.github.io/tree-sitter/) - Parser generator creating incremental parsers suitable for editor integration. Good example of modern parsing technology beyond traditional compiler construction.

* [ANTLR](https://www.antlr.org/) - Popular parser generator supporting multiple target languages. Extensive documentation and community resources.

* [Compiler Explorer](https://godbolt.org/) - Online tool for exploring compiler output across different compilers and optimization levels. Invaluable for understanding code generation.

* [ANTLRWorks](https://www.antlr3.org/works/) - GUI development environment for ANTLR grammars with visualization and debugging features.

## Community and Support

* [r/Compilers](https://www.reddit.com/r/Compilers/) - Active subreddit for compiler construction discussions, project showcases, and questions.

* [Compiler Jobs](https://mgaudet.github.io/CompilerJobs/) - Matthew Gaudet's curated list of compiler engineering positions.

* [LLVM Discourse](https://discourse.llvm.org/) - Official LLVM community forum for discussions about LLVM, Clang, and related projects.

* [Rust Compiler Team](https://www.rust-lang.org/governance/teams/compiler) - Information about contributing to the Rust compiler and joining the community.
