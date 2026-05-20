# marginalia

The `marginalia` crate is a trivia-preserving adapter that slots into the standard `logos` + `lalrpop` pipeline. Most parsers drop comments and blank lines at the lexer stage, which is fine for an interpreter but ruinous for a formatter, a refactoring tool, or any other consumer that has to round-trip back to source. `marginalia` solves this by wrapping a `logos` iterator in an iterator adapter that strips comment tokens from the stream the parser sees while recording them on the side, indexed by byte offset, so a later pass can re-attach them to AST nodes.

The crate is small on purpose. It does not parse, format, or know about your AST. It exposes one trait (`Classify`), one iterator (`TriviaLexer`), and a `TriviaTable` of events. A separate `attach` module re-anchors trivia onto AST spans; a separate `pretty` module is a `Doc`-style IR with trivia slots. Pick the pieces you need.

## Token Classification

The user's token enum implements `Classify` to tell `marginalia` which variants are trivia and which are not. Anything that returns `Some(TriviaPiece)` is recorded and stripped from the stream; everything else passes through unchanged.

```rust
#![enum!("marginalia/src/lib.rs", Tok)]
```

The two comment variants carry their lexeme so the `Classify` impl can hand the text straight to `marginalia`:

```rust,ignore
impl Classify for Tok {
    fn trivia(&self) -> Option<TriviaPiece<'_>> {
        match self {
            Self::LineComment(s) => Some(TriviaPiece { kind: TriviaKind::Line, text: s }),
            Self::BlockComment(s) => Some(TriviaPiece { kind: TriviaKind::Block, text: s }),
            _ => None,
        }
    }
}
```

`TriviaKind::Line` and `TriviaKind::Block` are the only two kinds; blank-line events are detected by the lexer itself by counting newlines between adjacent tokens, so the user does not have to model them.

## The Lexer Adapter

`TriviaLexer<I, T, E>` takes any `Iterator<Item = Result<(usize, T, usize), E>>` (the shape `lalrpop` expects) and yields the same shape, minus trivia. The constructor also borrows the source so it can detect blank-line runs by inspecting the bytes between tokens.

```rust
#![function!("marginalia/src/lib.rs", lex)]
```

The returned `Vec<TriviaEvent>` is what a downstream formatter consumes. Each event has a `Span { start, end }` and a `Trivia` variant (`Line`, `Block`, or `BlankLine`).

## Recovering the Trivia Table

Two ways to get the table back out: `table()` borrows it while the lexer is still in use; `into_table()` consumes the lexer. The composition pattern in the README chains `TriviaLexer` inside `offsides::LayoutLexer`, and the layout lexer exposes an `into_inner()` so the trivia table can still be recovered after both adapters are done.

```rust
#![function!("marginalia/src/lib.rs", describe)]
```

## Composition with offsides

`marginalia` and [`offsides`](./offsides.md) are designed to compose. Stack `TriviaLexer` _inside_ `LayoutLexer` so trivia is stripped before the layout algorithm starts measuring columns:

```rust
let raw = MyTok::lexer(source).spanned().map(/* shape into (lo, tok, hi) */);
let trivia = marginalia::TriviaLexer::new(raw, source);
let layout = offsides::LayoutLexer::new(trivia, source, cfg);
let ast = MyParser::new().parse(layout)?;
```

The grammar sees a stream that is both trivia-clean and braced-and-semicoloned, with neither concern leaking into the `.lalrpop` file.

## Best Practices

Keep `Classify::trivia` total over your enum: returning `None` for the non-trivia arms is required and the compiler will catch missing arms if you write an exhaustive `match`.

Capture comment text in the token variant. The `Classify` impl needs an `&str`, so the variant has to own the lexeme. Logos's `|l| l.slice().to_owned()` callback does this in one line.

Resolve trivia at AST-build time, not at parse time. `lalrpop` actions run with `(usize, lo, hi)` spans available; pair those with `TriviaTable::between(lo, hi)` to attach leading or trailing trivia to the node being built.

For formatters, never re-tokenize. The original `TriviaTable` already has every comment with its byte span; the `attach` and `pretty` modules are designed around that fact.
