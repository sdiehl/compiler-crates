# rustyline

The `rustyline` crate provides a pure-Rust readline implementation for building command-line interfaces. In compiler development, interactive REPLs (Read-Eval-Print Loops) are essential tools for testing language features, debugging compilation passes, and providing an interactive development environment. Rustyline offers features like line editing, history, completion, syntax highlighting, and multi-line input validation that make professional-quality REPLs possible.

A compiler REPL allows developers to experiment with language constructs, inspect intermediate representations, test type inference, and debug compilation errors interactively. Rustyline handles all the terminal interaction complexity, letting compiler authors focus on language semantics and compilation logic.

## Basic REPL Structure

Creating a compiler REPL starts with defining commands and setting up the editor:

```rust
#![function!("rustyline/src/lib.rs", create_editor)]
```

The configuration enables history tracking, list-style completions, and Emacs keybindings. The helper object provides all the advanced features like completion and syntax highlighting.

## Command System

A well-designed compiler REPL provides commands for various compilation stages:

```rust
#![struct!("rustyline/src/lib.rs", CompilerCommand)]
```

```rust
#![function!("rustyline/src/lib.rs", process_command)]
```

Commands allow users to load files, compile code, inspect ASTs and IR, query types, and manage the compilation context. This structure makes the REPL extensible and discoverable.

## Completion Support

Intelligent completion improves REPL usability significantly:

```rust
#![struct!("rustyline/src/lib.rs", CommandCompleter)]
```

```rust
impl Completer for CommandCompleter {
    type Candidate = Pair;
    
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>)> {
        let line_before_cursor = &line[..pos];
        let words: Vec<&str> = line_before_cursor.split_whitespace().collect();
        
        if words.is_empty() || (words.len() == 1 && !line_before_cursor.ends_with(' ')) {
            // Complete commands at start of line
            let prefix = words.get(0).unwrap_or(&"");
            let matches: Vec<Pair> = self.commands
                .iter()
                .filter(|cmd| cmd.starts_with(prefix))
                .map(|cmd| Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                })
                .collect();
            
            Ok((0, matches))
        } else {
            // Complete keywords within expressions
            let last_word = words.last().unwrap_or(&"");
            let word_start = line_before_cursor.rfind(last_word).unwrap_or(pos);
            
            let matches: Vec<Pair> = self.keywords
                .iter()
                .filter(|kw| kw.starts_with(last_word))
                .map(|kw| Pair {
                    display: kw.to_string(),
                    replacement: kw.to_string(),
                })
                .collect();
            
            Ok((word_start, matches))
        }
    }
}
```

The completer distinguishes between command completion (at the start of a line) and keyword completion (within expressions). This context-aware completion helps users discover commands and write code faster.

## Syntax Highlighting

Visual feedback through syntax highlighting makes the REPL more pleasant to use:

```rust
#![trait_impl!("rustyline/src/lib.rs", Highlighter for CompilerREPL)]
```

The highlighter colors commands differently from regular input and highlights matching brackets. This immediate visual feedback helps users spot syntax errors before execution.

## Input Validation

Multi-line input support requires validation to determine when input is complete:

```rust
#![struct!("rustyline/src/lib.rs", CompilerValidator)]
```

```rust
impl Validator for CompilerValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult> {
        let input = ctx.input();
        let mut stack = Vec::new();
        
        for ch in input.chars() {
            match ch {
                '(' | '{' | '[' => stack.push(ch),
                ')' => {
                    if stack.pop() != Some('(') {
                        return Ok(ValidationResult::Invalid(Some("Mismatched parentheses".into())));
                    }
                }
                '}' => {
                    if stack.pop() != Some('{') {
                        return Ok(ValidationResult::Invalid(Some("Mismatched braces".into())));
                    }
                }
                ']' => {
                    if stack.pop() != Some('[') {
                        return Ok(ValidationResult::Invalid(Some("Mismatched brackets".into())));
                    }
                }
                _ => {}
            }
        }
        
        if stack.is_empty() {
            Ok(ValidationResult::Valid(None))
        } else {
            Ok(ValidationResult::Incomplete)
        }
    }
}
```

The validator checks bracket matching to determine if more input is needed. This enables natural multi-line input for function definitions and complex expressions without requiring explicit continuation markers.

## Helper Integration

Rustyline uses a helper trait to combine all features:

```rust
#![struct!("rustyline/src/lib.rs", CompilerREPL)]
```

The helper struct implements all the necessary traits and maintains shared state like command definitions and configuration. This design keeps the implementation modular while providing a cohesive interface.

## Best Practices

Design commands that mirror your compiler's architecture. If your compiler has distinct phases like parsing, type checking, and code generation, provide commands to inspect the output of each phase. This helps users understand how their code flows through the compiler.

Implement context-aware completion that understands your language's syntax. Beyond simple keyword completion, consider completing function names, type names, and module paths based on the current compilation context. This requires integration with your compiler's symbol tables.

Use validation to support natural multi-line input for your language. If your language uses indentation or keywords to delimit blocks, implement validation logic that understands these patterns. Users should be able to paste multi-line code naturally.

Provide rich error formatting in the REPL. When compilation errors occur, format them with source context, underlining, and helpful messages. The immediate feedback of a REPL makes it ideal for learning a language.

Consider implementing a notebook mode that can save and replay REPL sessions. This is valuable for creating reproducible examples, tutorials, and bug reports. Store both input and output with enough context to replay the session.

Add introspection commands that leverage your compiler's internal representations. Commands to show type inference results, macro expansions, optimization decisions, and lowered code help users understand the compilation process.

The REPL can serve as more than just an interactive interpreter. It can be a powerful debugging and development tool that provides insight into every stage of compilation.
