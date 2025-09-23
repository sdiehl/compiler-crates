# miette

Miette is a comprehensive diagnostic library that brings Rust's excellent error reporting philosophy to your compiler projects. It provides a complete framework for creating beautiful, informative error messages with minimal boilerplate. The library excels at showing context, providing actionable help text, and maintaining consistency across all diagnostics.

Unlike simpler error reporting libraries, miette handles the entire diagnostic pipeline: from error definition through to rendering. It supports fancy Unicode rendering, screen-reader-friendly output, syntax highlighting, and even clickable error codes in supported terminals. The derive macro makes it trivial to create rich diagnostics while maintaining type safety.

## Core Concepts

Miette revolves around the Diagnostic trait, which extends the standard Error trait with additional metadata. Every diagnostic can include source code snippets, labeled spans, help text, error codes, and related errors. The library handles all the complexity of rendering these elements attractively.

```rust
#![struct!("miette/src/lib.rs", ParseError)]
```

This parse error demonstrates the key components: source code attachment, labeled spans with custom messages, error codes with documentation links, and contextual help. The `#[source_code]` attribute tells miette where to find the source text for snippet rendering.

## Type-Safe Diagnostics

Creating type-safe, reusable diagnostic types is straightforward with the derive macro. Each error type can capture all relevant context and provide specialized help based on the specific situation.

```rust
#![struct!("miette/src/lib.rs", TypeMismatchError)]
```

The type mismatch error shows how to provide contextual suggestions. By examining the expected and actual types, it can offer specific conversion advice. This pattern scales well to complex type systems with many possible conversions.

## Undefined Variables with Suggestions

One of miette's strengths is showing related information. The undefined variable error demonstrates how to include suggestions and point to similar names in scope.

```rust
#![struct!("miette/src/lib.rs", UndefinedVariableError)]
```

The `#[related]` attribute allows including sub-diagnostics that provide additional context. Each related diagnostic can have its own spans and messages, creating a rich, multi-layered error report.

## Collecting Multiple Errors

Real compilers often need to report multiple errors at once. Miette handles this elegantly through the related errors feature.

```rust
#![struct!("miette/src/lib.rs", CompilationErrors)]
```

This pattern allows accumulating errors during compilation and reporting them all together. The dynamic dispatch through `Box<dyn Diagnostic>` means you can mix different error types in the same collection.

## Borrow Checker Diagnostics

Complex diagnostics like borrow checker errors benefit from multiple labeled spans showing the relationship between different code locations.

```rust
#![struct!("miette/src/lib.rs", BorrowError)]
```

Multiple labels with different roles (primary vs secondary) help users understand the flow of borrows through their code. The optional spans allow for cases where some information might not be available.

## Pattern Matching Exhaustiveness

The collection label feature is perfect for showing multiple related locations, such as missing patterns in a match expression.

```rust
#![struct!("miette/src/lib.rs", NonExhaustiveMatch)]
```

The `#[label(collection)]` attribute works with any iterator of spans, making it easy to highlight multiple locations with similar issues.

## Integration with Standard Errors

Miette integrates seamlessly with Rust's error handling ecosystem. The `IntoDiagnostic` trait allows converting any standard error into a diagnostic, while the `Result` type alias provides ergonomic error handling.

```rust
#![function!("miette/src/lib.rs", create_diagnostic)]
```

This function shows how to create diagnostics dynamically when you don't know the error structure at compile time. It's useful for scripting languages or plugin systems where errors are defined at runtime.

## Screen Reader Support

Miette automatically detects when to use its screen-reader-friendly output format based on environment variables and terminal capabilities. This ensures your compiler is accessible to all users without additional configuration.

The narratable output format presents all the same information as the graphical format but in a linear, screen-reader-friendly way. Error codes become clickable links in terminals that support them, improving the documentation discovery experience.

## Best Practices

Structure your diagnostics hierarchically. Top-level errors should provide overview information, while related errors can provide specific details. This helps users understand both the big picture and the specifics.

Use error codes consistently and link them to documentation. The `url(docsrs)` shorthand automatically generates links to your docs.rs documentation, making it easy for users to find detailed explanations.

Provide actionable help text. Instead of just describing what went wrong, suggest how to fix it. Include example code in help messages when appropriate.

Keep source spans accurate. Miette's snippet rendering is only as good as the spans you provide. Take care to highlight exactly the relevant code, neither too much nor too little.

Use severity levels appropriately. Errors should block compilation, warnings should indicate potential issues, and notes should provide supplementary information. The fancy renderer uses different colors for each severity level.

For library code, always return concrete error types that implement Diagnostic. This gives consumers the flexibility to handle errors programmatically or render them with miette. Application code can use the more convenient `Result` type alias and error conversion utilities.

Miette has become essential infrastructure for Rust projects that prioritize user experience. Its thoughtful design and comprehensive features make it possible to create compiler diagnostics that genuinely help users understand and fix problems in their code.