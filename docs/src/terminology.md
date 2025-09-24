# Terminology

So you want to build a new compiler? Building a compiler is one of the most challenging and rewarding projects you can undertake. Some compilers rival operating systems in their complexity, but the journey of creating one provides deep insights into how programming languages work at their most fundamental level.

Building a compiler might be the right choice when a compiler for your desired language and target platform simply doesn't exist. Perhaps you're creating a domain-specific language for your industry, targeting unusual hardware, or implementing a new programming paradigm. Beyond practical needs, compiler development is profoundly educational. You'll gain intimate knowledge of how kernels, compilers, and runtime libraries interact, and you'll understand what it truly takes to implement a programming language. Is it strictly neccessary to learn? No. But so few things in life are.

However, compiler development demands significant knowledge. You need complete understanding of your source language specification and deep knowledge of your target architecture's assembly language. Creating a production-quality compiler that rivals GCC or LLVM in optimization capabilities requires years (if not decades) of dedicated work. Full compliance with language specifications proves surprisingly difficult, as edge cases and subtle interactions between features often reveal themselves only during implementation.

Understanding compiler terminology helps navigate the field's extensive literature. The **host** system runs the compiler itself, while the **target** system runs the compiled programs. When host and target match, you produce native executables. When they differ, you've built a cross-compiler. The **runtime** encompasses libraries and processes available on the target system that programs depend on. Two machines with identical hardware but different runtime resources effectively become different targets.

An **executable** contains all information necessary for the target system to launch your program. While it could be a flat binary, modern executables include linking information, relocation data, and metadata. The **linker** creates connections between your program and the runtime it depends on. Programs without these dependencies, like operating system kernels, are called freestanding. A compiler capable of compiling its own source code is **self-hosting**, representing a significant milestone in compiler maturity.

Modern compilers divide their work into distinct phases, each handling specific transformation tasks. This modular architecture enables code reuse and simplifies development. The standard pipeline consists of three major components working in sequence.

The **front end** accepts source code in a specific programming language and transforms it into an **intermediate representation** (or IR for short). This phase handles all language-specific processing including parsing syntax, checking types, and resolving names. By producing a common IR, front ends for different languages can share the same optimization and code generation infrastructure.

The **middle end** operates on the intermediate representation to improve code quality. This optional phase applies optimization algorithms that eliminate redundancy, improve performance, and reduce code size. Because it works with abstract IR rather than source code or machine code, optimizations implemented here benefit all source languages and target architectures.

The **back end** consumes intermediate representation and produces executable code for specific target architectures. This phase handles machine-specific concerns like register allocation, instruction selection, and executable file format generation. By separating target-specific code into the back end, the same IR can be compiled for different architectures without modifying earlier phases.

## Front End Components

The front end transforms human-readable source code into a form suitable for analysis and optimization. After accepting files and processing command-line options through its user interface, the front end processes code through several stages.

A **preprocessor** handles textual transformations before compilation begins. In C-like languages, this includes copying header file contents, expanding macros, and processing conditional compilation directives. The preprocessor works purely with text, unaware of the language's actual syntax.

The **scanner** (or **lexer**) reads preprocessed source text and produces a stream of tokens representing the language's basic vocabulary. Each identifier, keyword, operator, and literal becomes a discrete token. The scanner handles details like recognizing number formats, processing string escape sequences, and skipping whitespace.

The **parser** consumes tokens and constructs a tree structure representing the program's syntactic structure. This parse tree captures how tokens group into expressions, statements, and declarations according to the language grammar. Modern parsers often build more abstract representations that omit syntactic noise like parentheses and semicolons.

The **semantic analyzer** traverses the parse tree to determine meaning. It builds symbol tables mapping names to their declarations, checks that types match correctly, and verifies that the program follows all language rules not captured by the grammar. This phase transforms a syntactically valid program into a semantically valid one.

A **type checker** is often an integral part of semantic analysis. It ensures that operations are applied to compatible types, infers types where possible, and enforces language-specific type rules. Type checking can be simple in dynamically-typed languages or complex in statically-typed languages with features like generics and polymorphism. See my [Typechecker Zoo](https://sdiehl.github.io/typechecker-zoo/) writeup for more details on this phase.

Finally, the front end generates its **intermediate representation**. A well-designed IR captures all source program semantics while adding explicit information about types, control flow, and data dependencies that later phases need. The IR might resemble the parse tree, use three-address code, employ static single assignment form, or adopt more exotic representations.

## Middle End Processing

The middle end hosts numerous optimization passes that improve program performance without changing its observable behavior. These optimizations work at various granularities from individual instructions to whole-program transformations.

Common optimizations include dead code elimination to remove unreachable or unnecessary code, constant propagation to replace variables with known values, and loop optimizations that reduce iteration overhead. More sophisticated techniques like inlining, vectorization, and escape analysis require complex analysis but can dramatically improve performance.

The middle end also performs essential transformations even in unoptimized builds. These include lowering high-level constructs to simpler operations and inserting runtime checks required by the language specification. While you might omit sophisticated optimizations in a simple compiler, some middle-end processing often proves necessary.

## Back End Components

The back end bridges the gap between abstract intermediate representation and concrete machine code. This phase handles all target-specific details that earlier phases deliberately ignored.

The **code generator** traverses the IR and emits assembly-like instructions. To simplify this process, it typically assumes unlimited registers and ignores calling conventions initially. This pseudo-assembly captures the desired computation without committing to specific resource allocation decisions.

The **register allocator** maps the code generator's virtual registers onto the limited physical registers available on the target CPU. This critically important phase uses sophisticated algorithms to minimize memory traffic by keeping frequently-used values in registers. Poor register allocation can devastate performance.

The **assembler** translates assembly language into machine code bytes. It encodes each instruction according to the target architecture's rules and tracks addresses of labels for jump instructions. The assembler produces object files containing machine code plus metadata about symbols and relocations.

The **linker** combines object files with libraries to create complete executables. It resolves references between compilation units, performs relocations to assign final addresses, and adds runtime loader information. While sometimes considered separate from compilation, linking remains essential for producing runnable programs.

Remember that compiler construction is as much an empirical engineering discipline as theoretical knowledge. Start simple, test thoroughly, and gradually add complexity. Even a basic compiler that handles a subset of a language provides valuable learning experiences.

But we live in great times where much of the complexity has been abstracted into off-the-shelf libraries. The crates covered here give you professional-quality tools to build upon, letting you focus on the interesting problems unique to your language and target platform. It's never been a better time to be a compiler developer.
