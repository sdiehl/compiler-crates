# Parser Comparison

This page provides a comprehensive comparison of actual parser generators and parser combinator libraries covered in this guide. Each parser uses different parsing algorithms and techniques, making them suitable for different language implementation scenarios.

## Legend

- 🟢 Full support
- 🟡 Partial support or with limitations
- 🔴 Not supported

## Parser Overview

| Parser      | Type              | Algorithm         | Grammar Format            | Performance | Error Recovery | Learning Curve | Production Ready | Best For                                    |
| ----------- | ----------------- | ----------------- | ------------------------- | ----------- | -------------- | -------------- | ---------------- | ------------------------------------------- |
| **nom**     | Parser Combinator | Recursive Descent | Rust combinators          | ⭐⭐⭐⭐⭐  | ⭐⭐           | ⭐⭐⭐⭐       | ⭐⭐⭐⭐⭐       | Binary formats, streaming protocols         |
| **pest**    | Parser Generator  | PEG               | External `.pest` files    | ⭐⭐⭐      | ⭐⭐⭐⭐       | ⭐⭐           | ⭐⭐⭐⭐         | Prototyping, DSLs, configuration languages  |
| **lalrpop** | Parser Generator  | LALR(1)           | External `.lalrpop` files | ⭐⭐⭐⭐⭐  | ⭐⭐⭐         | ⭐⭐⭐⭐       | ⭐⭐⭐⭐⭐       | Production compilers, programming languages |
| **chumsky** | Parser Combinator | Recursive Descent | Rust combinators          | ⭐⭐⭐      | ⭐⭐⭐⭐⭐     | ⭐⭐           | ⭐⭐⭐           | Error recovery, IDE support                 |
| **winnow**  | Parser Combinator | Recursive Descent | Rust combinators          | ⭐⭐⭐⭐⭐  | ⭐⭐           | ⭐⭐⭐         | ⭐⭐⭐⭐         | Successor to nom, cleaner API               |
| **pom**     | Parser Combinator | Recursive Descent | Rust combinators          | ⭐⭐⭐⭐    | ⭐⭐           | ⭐⭐⭐         | ⭐⭐⭐           | Simple parsers, educational                 |

## Parsing Algorithm Characteristics

| Algorithm             | Left Recursion        | Ambiguity           | Backtracking | Memory Usage | Parse Time                | Lookahead |
| --------------------- | --------------------- | ------------------- | ------------ | ------------ | ------------------------- | --------- |
| **LALR(1)**           | 🟢 Handles naturally  | 🔴 Must resolve     | 🔴 None      | Low          | O(n)                      | 1 token   |
| **PEG**               | 🔴 Requires rewriting | 🟢 First match wins | 🟢 Unlimited | Medium       | O(n) typical, O(n²) worst | Unlimited |
| **Recursive Descent** | 🔴 Stack overflow     | 🟢 Can handle       | 🟢 Manual    | Low          | O(n) to O(n²)             | Unlimited |

## Detailed Feature Comparison

| Feature                    | nom      | pest     | lalrpop      | chumsky  | winnow   | pom     |
| -------------------------- | -------- | -------- | ------------ | -------- | -------- | ------- |
| **Grammar Definition**     |          |          |              |          |          |         |
| External grammar files     | 🔴       | 🟢       | 🟢           | 🔴       | 🔴       | 🔴      |
| Inline in Rust code        | 🟢       | 🔴       | 🔴           | 🟢       | 🟢       | 🟢      |
| Type-safe                  | 🟢       | 🟡       | 🟢           | 🟢       | 🟢       | 🟢      |
| Grammar validation         | Runtime  | Runtime  | Compile-time | Runtime  | Runtime  | Runtime |
| **Parsing Features**       |          |          |              |          |          |         |
| Streaming input            | 🟢       | 🔴       | 🔴           | 🟡       | 🟢       | 🔴      |
| Zero-copy parsing          | 🟢       | 🟢       | 🟢           | 🟡       | 🟢       | 🟢      |
| Incremental parsing        | 🔴       | 🔴       | 🔴           | 🔴       | 🔴       | 🔴      |
| Memoization/Packrat        | 🔴       | 🟢       | 🔴           | 🟡       | 🔴       | 🟡      |
| Custom lexer support       | 🟢       | N/A      | 🟢           | 🟢       | 🟢       | 🟢      |
| **Error Handling**         |          |          |              |          |          |         |
| Error recovery             | 🔴       | 🟢       | 🟡           | 🟢       | 🔴       | 🔴      |
| Custom error types         | 🟢       | 🟢       | 🟢           | 🟢       | 🟢       | 🟢      |
| Error position tracking    | 🟢       | 🟢       | 🟢           | 🟢       | 🟢       | 🟢      |
| Multiple errors            | 🔴       | 🟢       | 🔴           | 🟢       | 🔴       | 🔴      |
| Contextual errors          | 🟢       | 🟢       | 🟡           | 🟢       | 🟢       | 🟡      |
| **AST Generation**         |          |          |              |          |          |         |
| Automatic AST generation   | 🔴       | 🟡       | 🟢           | 🔴       | 🔴       | 🔴      |
| Custom AST types           | 🟢       | 🟢       | 🟢           | 🟢       | 🟢       | 🟢      |
| Location spans             | 🟢       | 🟢       | 🟢           | 🟢       | 🟢       | 🟢      |
| **Development Experience** |          |          |              |          |          |         |
| IDE support                | 🟡       | 🟡       | 🟡           | 🟡       | 🟡       | 🟡      |
| Debugging tools            | 🟡       | 🟢       | 🟢           | 🟢       | 🟡       | 🟡      |
| Documentation quality      | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐       | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐  |

## Grammar Complexity Support

| Feature               | nom    | pest | lalrpop | chumsky | winnow | pom    |
| --------------------- | ------ | ---- | ------- | ------- | ------ | ------ |
| **Grammar Types**     |        |      |         |         |        |        |
| Context-free          | 🟢     | 🟢   | 🟢      | 🟢      | 🟢     | 🟢     |
| Context-sensitive     | 🟢     | 🔴   | 🔴      | 🟡      | 🟢     | 🟡     |
| Ambiguous grammars    | 🟢     | 🟡   | 🔴      | 🟢      | 🟢     | 🟢     |
| **Advanced Features** |        |      |         |         |        |        |
| Left recursion        | 🟡*    | 🔴   | 🟢      | 🔴      | 🟡*    | 🔴     |
| Operator precedence   | Manual | 🟢   | 🟢      | 🟢      | Manual | Manual |
| Parameterized rules   | 🟢     | 🔴   | 🟢      | 🟢      | 🟢     | 🟢     |
| Semantic predicates   | 🟢     | 🟢   | 🔴      | 🟢      | 🟢     | 🟢     |

*Can be handled with special combinators or techniques

## tl;dr Recommendations

### Choose **nom** when:

- Parsing binary formats or network protocols
- Need streaming/incremental parsing
- Performance is critical
- Want fine-grained control over parsing

### Choose **pest** when:

- Rapid prototyping of new languages
- Grammar readability is important
- Need good error messages out of the box
- Working with configuration languages or DSLs

### Choose **lalrpop** when:

- Building production programming language compilers
- Grammar has left recursion
- Need maximum parsing performance
- Want compile-time grammar validation

### Choose **chumsky** when:

- Error recovery is critical (IDE/LSP scenarios)
- Need excellent error messages
- Building development tools
- Want modern combinator API

### Choose **winnow** when:

- Starting a new project (nom successor)
- Want cleaner API than nom
- Need streaming support
- Performance is important

### Choose **pom** when:

- Learning parser combinators
- Building simple parsers
- Want minimal dependencies
- Prefer simple, readable code

In short:

- **For maximum performance**: LALRPOP (with grammar restrictions) or nom/winnow (with flexibility)
- **For best developer experience**: pest (external grammars) or chumsky (error recovery)
- **For binary formats**: nom or winnow are specifically designed for this
- **For production compilers**: LALRPOP provides the traditional compiler construction approach
- **For learning**: pom offers the simplest mental model

Each parser makes different trade-offs between performance, expressiveness, error handling, and ease of use. Consider your specific requirements carefully when making a selection.
