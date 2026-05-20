# offsides

The `offsides` crate is a layout-sensitive lexer adapter for the `logos` + `lalrpop` stack. It implements the off-side rule (Landin, 1966): the algorithm behind Haskell's implicit `do`/`where`/`let`/`of` blocks, Python's INDENT/DEDENT, and PureScript / Elm / Idris layout. Languages that use indentation to delimit blocks have historically each hand-rolled this in C or in their own front end; `offsides` extracts it as a single configurable iterator adapter so a downstream `lalrpop` grammar can pretend the source had real braces and semicolons.

The library is opinionated about staying out of your way. It does not own your token enum, does not name virtual tokens for you, and is not hard-coded to any one language. You implement a three-method `Layout` trait on whatever enum you already have, pass a function pointer that decides which real tokens open a new block, and the adapter splices `v_open()`, `v_sep()`, and `v_close()` tokens at the right byte offsets.

## The Layout Trait

`Layout` is the only trait the consumer implements. Three constructors. The names of the virtual variants belong to the user.

```rust
#![enum!("offsides/src/lib.rs", Tok)]
```

```rust,ignore
impl Layout for Tok {
    fn v_open() -> Self { Self::VOpen }
    fn v_close() -> Self { Self::VClose }
    fn v_sep() -> Self { Self::VSemi }
}
```

Which _real_ tokens open a layout block is a runtime predicate, not a trait method, so the same enum can drive different rules in different contexts (e.g. a REPL with no opener vs a module file with several).

```rust
#![function!("offsides/src/lib.rs", is_opener)]
```

## Lazy Mode (Haskell, ML, PureScript, Elm, Idris)

In `Lazy` mode a layout block only opens after an opener keyword. Top-level tokens stay un-bracketed. This is the Haskell-style algorithm: `do { ... }` shape, but the braces are implicit.

```rust
#![function!("offsides/src/lib.rs", lex)]
```

Given the input

```
let
  x = 1
  y = 2
in x + y
```

the adapter yields `Let VOpen Ident("x") Eq Num(1) VSemi Ident("y") Eq Num(2) VClose In Ident("x") Plus Ident("y")`. The grammar matches `"v{"`, `"v;"`, `"v}"` exactly as it would match real punctuation.

## Eager Mode (Python)

In `Eager` mode the very first token starts the top-level layout block; every subsequent indent change emits virtual tokens. There is no opener keyword to wait for.

```rust
#![function!("offsides/src/lib.rs", lex_eager)]
```

The same code path serves both modes; `LayoutMode::Eager` is a single config flag.

## Algorithm

For each inner token `(lo, tok, hi)` with column `col = column_of(lo)`:

1. If the previous emitted token was an opener, push `col` onto the indent stack and yield `v_open()` first.
2. If `lo` is on a later line than the previous token's `hi`, pop any frames with `top > col` (yielding `v_close()` each) and emit `v_sep()` if `top == col`.
3. Emit the real token.
4. If `is_opener(tok)`, arm step 1 for the next token.
5. At EOF, drain the stack with `v_close()` per remaining frame.

Virtual tokens carry zero-width spans (`lo == hi`) at the boundary they were inserted at, which keeps downstream error reporting pointing at real source positions.

## Composition with marginalia

Stack [`marginalia::TriviaLexer`](./marginalia.md) _inside_ `LayoutLexer` so comments are stripped before the layout algorithm starts measuring columns; otherwise a leading comment on a continuation line will skew the indent decision.

```rust
let raw = MyTok::lexer(source).spanned().map(/* shape into (lo, tok, hi) */);
let trivia = marginalia::TriviaLexer::new(raw, source);
let layout = offsides::LayoutLexer::new(trivia, source, cfg);
let ast = MyParser::new().parse(layout)?;
let (_, table) = layout.into_inner().into_parts();
```

The grammar sees a clean braced and semicoloned stream; the trivia table is still recoverable for a formatter pass.

## Best Practices

Use `fn(&T) -> bool` for predicates, not `Box<dyn Fn(&T) -> bool>`. `LayoutConfig` is parameterised on function pointers so the consumer pays zero allocation and zero indirection per token.

Strip trivia before layout. A `// trailing comment` on a continuation line will not break the off-side rule because `marginalia` is upstream and has already removed it from the column the layout adapter sees.

Use `LayoutMode::Eager` for Python-shaped languages and `LayoutMode::Lazy` for everything else. The default is `Lazy`.

For languages that allow `{ ... }` to escape the off-side rule (Haskell does), enable `LayoutConfig::with_explicit_braces` so the adapter suspends and resumes layout around the explicit pair.

Default `tab_width` is 1. The algorithm only needs ordinal column comparisons; it never compares to display width. Set a larger value only if the language specifies one.
