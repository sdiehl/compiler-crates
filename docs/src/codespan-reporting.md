# codespan-reporting

The codespan-reporting crate provides beautiful diagnostic rendering for compilers and development tools. It generates the same style of error messages you see in Rust, with source code snippets, underlines, and helpful annotations. This has become the standard for high-quality compiler diagnostics in the Rust ecosystem.

Unlike simple error printing, codespan-reporting handles multi-line spans, multiple files, and complex error relationships. It automatically handles terminal colors, Unicode rendering, and even provides alternatives for non-Unicode terminals. The crate integrates seamlessly with language servers and other tooling.

## Core Concepts

The library revolves around three main types: Files for source management, Diagnostics for error information, and Labels for marking specific code locations. Each diagnostic can have multiple labels pointing to different parts of the code, making it easy to show cause-and-effect relationships.

```rust
#![struct!("codespan-reporting/src/lib.rs", DiagnosticEngine)]
```

This wrapper provides a convenient interface for managing files and emitting diagnostics. In a real compiler, you would integrate this with your existing source management system.

## Creating Diagnostics

Diagnostics are built using a fluent API that makes it easy to construct rich error messages. Each diagnostic has a severity level, a main message, and can include multiple labeled spans with their own messages.

```rust
#![enum!("codespan-reporting/src/lib.rs", CompilerError)]
```

The `to_diagnostic` method converts these error types into codespan-reporting diagnostics with appropriate labels and notes.

The type error case shows how to create a diagnostic with primary and secondary labels. The primary label marks the error location, while secondary labels provide additional context.

## File Management

For multi-file projects, codespan-reporting provides a simple file database interface. You can use the built-in SimpleFiles or implement your own file provider.

```rust
#![struct!("codespan-reporting/src/lib.rs", Project)]
```

The Project struct demonstrates how to manage multiple source files and emit diagnostics across them. This is essential for real compilers that need to report errors spanning multiple modules.

## Error Types

Well-designed error types make diagnostics more maintainable and consistent. Each error variant should capture all the information needed to generate a helpful diagnostic.

```rust
#![enum!("codespan-reporting/src/lib.rs", CompilerError)]
```

These error types demonstrate common patterns: type mismatches with expected and actual types, undefined variables with spelling suggestions, and parse errors with recovery hints.

## Advanced Features

The library supports warning and informational diagnostics, not just errors. Different severity levels help users prioritize what to fix first.

```rust
#![function!("codespan-reporting/src/lib.rs", create_warning)]
```

Warnings can include notes with additional context or suggestions for fixing the issue. This helps guide users toward better code patterns.

## Integration with Lexers

Real compiler diagnostics need accurate source locations. Here's a simple lexer that tracks positions for error reporting:

```rust
#![struct!("codespan-reporting/src/lib.rs", Lexer)]
```

The lexer maintains byte positions for each token, which can be used directly in diagnostic labels. This ensures error underlines appear in exactly the right place.

## Terminal Configuration

The library provides fine-grained control over output formatting through the Config type. You can customize colors, character sets, and layout options to match your project's needs. The default configuration works well for most cases, automatically adapting to terminal capabilities.

## Best Practices

Structure your error types to capture semantic information, not just strings. This makes it easier to provide consistent, helpful diagnostics throughout your compiler. Include spelling suggestions when reporting undefined names by computing edit distance to known identifiers.

Group related errors together when they have a common cause. For example, if a type error cascades through multiple expressions, show the root cause prominently and list the consequences as secondary information.

Use notes and help messages to educate users about language features. Good diagnostics teach users how to write better code, not just point out what's wrong. Include examples in help text when appropriate.

For parse errors, show what tokens were expected at the error location. This helps users understand the grammar and fix syntax errors quickly. Recovery hints can suggest common fixes for typical mistakes.

The codespan-reporting crate has become essential infrastructure for Rust compiler projects. Its thoughtful design and attention to user experience set the standard for compiler diagnostics. By following its patterns, you can provide error messages that help rather than frustrate your users.