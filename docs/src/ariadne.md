# ariadne

Ariadne is a modern diagnostic reporting library that emphasizes beautiful, user-friendly error messages. Named after the Greek mythological figure who provided thread to navigate the labyrinth, ariadne helps users navigate through complex error scenarios with clear, visually appealing diagnostics. It provides more flexibility and features than codespan-reporting while maintaining ease of use.

The library excels at showing relationships between different parts of code, using colors and connecting lines to make error contexts immediately clear. It supports multi-file errors, complex label relationships, and provides excellent defaults while remaining highly customizable.

## Core Architecture

Ariadne separates error data from presentation, making it easy to generate diagnostics from your existing error types. The Report type is the centerpiece, built using a fluent API that encourages clear, helpful error messages.

```rust
#![enum!("ariadne/src/lib.rs", CompilerDiagnostic)]
```

Each diagnostic variant captures semantic information about the error, not just locations and strings. This separation makes it easier to maintain consistent error messages and potentially provide automated fixes.

## Building Reports

Reports are constructed using a builder pattern that makes the intent clear and the code readable. Each report has a kind (error, warning, or advice), a main message, and can include multiple labels with different colors and priorities.

The `to_report` method on CompilerDiagnostic converts error data into ariadne Report objects, handling all the details of label creation and color assignment.

The type error case demonstrates several key features: colored type names for clarity, primary and secondary labels to show relationships, and helpful notes explaining why the error occurred.

## Color Management

Ariadne provides intelligent color assignment through ColorGenerator, ensuring that related labels have distinct, readable colors. This is especially useful for complex errors with many related locations.

```rust
#![function!("ariadne/src/lib.rs", error_report)]
```

These helper functions provide a simpler way to create basic error and warning reports when you don't need the full flexibility of the builder pattern.

The builder pattern allows for flexible report construction while ensuring all required fields are provided. This makes it harder to accidentally create incomplete error messages.

## Multi-file Errors

Modern compilers often need to report errors spanning multiple files. Ariadne handles this elegantly, allowing labels to reference different sources while maintaining a cohesive error presentation.

The CyclicDependency variant in CompilerDiagnostic shows how to represent errors involving multiple files. Each module in the cycle gets its own label with a distinct color, making the relationship clear.

## Source Management

For production use, you'll need a source management system that can efficiently provide file contents for error reporting. Ariadne works with any source provider through a simple trait.

```rust
#![struct!("ariadne/src/lib.rs", SourceManager)]
```

This manager can be extended to support incremental updates, caching, and other optimizations needed for language server implementations.

## Advanced Formatting

Ariadne supports rich formatting within messages using the Fmt trait. This allows for inline styling of important elements like type names, keywords, or suggestions.

The library provides extensive configuration options through the Config type. You can control character sets (Unicode vs ASCII), compactness, line numbering style, and more. This ensures your diagnostics look good in any environment.

## Language Server Integration

Ariadne diagnostics can be converted to Language Server Protocol format for IDE integration. This allows the same error reporting logic to power both command-line tools and IDE experiences.

```rust
#![function!("ariadne/src/lib.rs", to_lsp_diagnostic)]
```

The conversion preserves error codes, severity levels, and related information, ensuring a consistent experience across different tools.

## Best Practices

Design your error types to capture intent, not just data. Instead of a generic "SyntaxError", have specific variants like "MissingClosingBrace" or "UnexpectedToken". This makes it easier to provide targeted help.

Use color meaningfully. Primary error locations should use red, secondary related locations can use blue or yellow, and informational labels can use gray. Consistency helps users quickly understand error relationships.

Write error messages that teach. Instead of just saying what's wrong, explain why it's wrong and how to fix it. Good diagnostics are an opportunity to educate users about language rules and best practices.

Consider error recovery when designing diagnostics. If you can guess what the user meant, include that in help text. For example, if they typed "fucntion" instead of "function", suggest the correction.

Group related errors when they have a common cause. If a type error in one function causes errors in its callers, present them as a single diagnostic with multiple labels rather than separate errors.

The combination of clear visual design and thoughtful error messages makes ariadne an excellent choice for modern compiler projects. Its focus on user experience aligns well with Rust's philosophy of helpful error messages.