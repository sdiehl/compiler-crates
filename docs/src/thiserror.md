# thiserror

The `thiserror` crate is the standard tool for defining structured error enums in Rust libraries. It provides a derive macro that generates `Display` and `Error` impls from a single `#[error("...")]` attribute per variant, eliminating the boilerplate that hand-written error types normally require. Compilers use `thiserror` heavily because each phase (lex, parse, name resolution, type check, codegen) needs its own well-typed error vocabulary, and `thiserror` lets you express that vocabulary without writing reams of trait impls by hand.

The pure-`thiserror` approach has one ergonomic gap: there is no built-in way to attach free-form context as an error bubbles up. `anyhow` provides that style (`.context()`, `.with_context()`) but at the cost of type erasure, which makes pattern-matching impossible. The `thiserror-context` companion crate bridges the two: it wraps a `thiserror` enum in a context-carrying envelope, so you keep the strong typing of `thiserror` _and_ the layered context messages of `anyhow`. The example here uses both crates together.

## Phase-Local Error Enums

Each compiler phase defines its own `thiserror` enum. These look exactly like any other `thiserror` derivation: a `#[derive(Error, Debug)]` with `#[error("...")]` on each variant. `thiserror-context` does not require any change here.

```rust
#![enum!("thiserror/src/lib.rs", LexerErrorInner)]
```

```rust
#![enum!("thiserror/src/lib.rs", ParserErrorInner)]
```

```rust
#![enum!("thiserror/src/lib.rs", CompilerErrorInner)]
```

The `Inner` suffix is a convention: the _inner_ enum is the plain `thiserror` type, and a companion _outer_ wrapper carries the context. The compile-pipeline shape is the usual one: each phase has its own error vocabulary, and the top-level `CompilerError` enum wraps whichever phase failed.

## Wrapping with `impl_context!`

`impl_context!(Outer(Inner))` generates a context-carrying wrapper around a plain `thiserror` enum. The wrapper is itself an enum with two variants (a `Base` arm holding the original error and a `Context { error, context }` arm that carries a free-form message). The macro also implements `Display`, `Debug` (with `anyhow`-style "Caused by:" formatting), `Error`, `AsRef<Inner>`, and a blanket conversion from anything that converts into the inner type.

```rust,ignore
impl_context!(LexerError(LexerErrorInner));
impl_context!(ParserError(ParserErrorInner));
impl_context!(CompilerError(CompilerErrorInner));
```

After this, `LexerError` is the type you actually return from lexer functions; `LexerErrorInner` is the type you construct when raising a fresh error.

## Adding Context

The generated `Context` trait gives `.context(msg)` and `.with_context(|| msg)` extension methods on any `Result<_, Inner>`. Each call wraps the existing error in a new `Context` layer; the original variant stays untouched and remains pattern-matchable.

```rust
#![function!("thiserror/src/lib.rs", lex_identifier)]
```

`.context()` takes any `Display`-able value eagerly; `.with_context()` takes a closure that runs only on the error path (use the latter when building the context string is non-trivial).

## Cross-Phase Conversion with Context Carried

The headline feature is `impl_from_carry_context!(Source, Target, TargetVariant)`. It generates `impl From<Source> for Target` that walks every context layer on the source error, wraps the bare inner into the target variant, then re-applies the context layers in order. The result is that a `LexerError` with three context layers becomes a `ParserError` with the _same_ three context layers plus whatever the parser adds on top.

```rust,ignore
impl_from_carry_context!(LexerError, ParserError, ParserErrorInner::Lexer);
impl_from_carry_context!(LexerError, CompilerError, CompilerErrorInner::Lexer);
impl_from_carry_context!(ParserError, CompilerError, CompilerErrorInner::Parser);
```

The variant in `ParserErrorInner::Lexer(LexerError)` must _not_ be marked `#[from]`: the macro provides the `From` impl itself, with the context-walking behavior baked in. Using `#[from]` would conflict.

```rust
#![function!("thiserror/src/lib.rs", parse_let_binding)]
```

Note the `lex_identifier(name, 0)?` line: `?` auto-converts `LexerError` to `ParserError` through the bridge, preserving every context layer the lexer attached. Explicit `.with_context(...)` is omitted at that line because the `Context` trait impl becomes ambiguous when the current error type can convert into more than one wrapper. The fix is to add context calls only on freshly-constructed inner errors (where the source type is unambiguous) and rely on `?` for cross-phase conversion.

## Pipeline Composition

The top-level `compile` function ties the phases together. Once the error type is converted to the top-level `CompilerError` (which has no further conversion targets), `.context()` and `.with_context()` are unambiguous again.

```rust
#![function!("thiserror/src/lib.rs", compile)]
```

Running this on bad input prints the full context chain:

```text
Display: parser phase failed
Debug:
Parser(Lexer(UnexpectedChar { ch: '1', pos: 0 }))

Caused by:
    0: source: "let 1bad = x"
    1: compiling input
    2: identifier must start with a letter or underscore
```

The variant constructor (`Parser(Lexer(UnexpectedChar { ... }))`) is intact, ready to match on. The "Caused by:" section shows every `.context()` call along the way, deepest first.

## Downcasting with `as_ref()`

The wrapper's `AsRef<Inner>` impl peels back through any number of context layers to expose the original `thiserror` enum. This is how callers pattern-match on the underlying variant without caring about the context wrapping.

```rust,ignore
match err.as_ref() {
    ParserErrorInner::UnexpectedToken { expected, .. } => { /* ... */ }
    ParserErrorInner::MissingItem { item } => { /* ... */ }
    ParserErrorInner::Lexer(_) => { /* ... */ }
}
```

This is the discipline `anyhow` cannot offer: typed pattern-matching survives the contextualization.

## Best Practices

Add context at the point of construction, before wrapping. The `Context` trait blanket impl picks the target wrapper from `E: Into<Outer>`. If the current error type can convert into more than one wrapper (which happens once you have a top-level error envelope), `.context()` becomes ambiguous and the compiler will reject the call. Keep `.context()` calls on `Result<_, FooInner>` (one unique `Into` target) or on the top-level type (no further conversion targets), and use `?` for everything in between.

Do not combine `#[from]` with `impl_from_carry_context!` on the same variant. The macro provides its own `From` impl with context-walking; `#[from]` would conflict.

Prefer the `Inner`/wrapper naming convention. `LexerErrorInner` is what you `Err(...)` with; `LexerError` is what your function returns. Mixing these up will produce confusing inference errors.

For deep pipelines, expose only the top-level `CompilerError` in your public API. Internal phase types stay private; conversions happen at function boundaries via `?`.

Use the `Debug` impl, not `Display`, for end-user error reporting. The `Debug` impl emits the "Caused by:" cascade; `Display` shows only the innermost message.
