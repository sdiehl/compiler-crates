# unbound

The `unbound` crate handles variable binding for abstract syntax trees. Every typechecker and evaluator for a language with lambdas eventually needs capture-avoiding substitution and alpha equivalence, and getting either wrong produces the most confusing bugs in a compiler. `unbound` derives both from the shape of your AST, in the spirit of the Haskell library of the same name.

Terms are stored in locally nameless form. Free variables carry a name with a globally unique index. Bound variables are de Bruijn coordinates, installed when a `Bind` closes over its body and removed again when the binder is opened. A bound variable has no name, so alpha equivalence is structural equality and substitution cannot capture.

## Terms

A binder is written `Bind<P, T>` where `P` is the pattern and `T` is the body. The derives generate `Alpha` and `Subst` by walking the structure. The variant named `Var` is the variable case.

```rust
#![enum!("unbound/src/lib.rs", Expr)]
```

`Name<Expr>` carries a phantom type, so a term variable can never be substituted into a type or the other way round. Smart constructors keep the examples readable.

```rust
#![function!("unbound/src/lib.rs", var)]
```

```rust
#![function!("unbound/src/lib.rs", lam)]
```

`Name::global` returns the same name for every call with the same spelling. Building a term bottom-up with global names gives every occurrence its nearest enclosing binder, which is exactly lexical scope. A parser can use this directly and skip a separate renaming pass.

## Alpha Equivalence

`\x. x` and `\y. y` are both stored as `\. #0`, so `aeq` is a lockstep walk with no renaming context.

```rust,ignore
assert!(lam("x", var("x")).aeq(&lam("y", var("y"))));
assert!(!lam("x", var("y")).aeq(&lam("y", var("y"))));
```

## Normalization

`unbind` opens a binder with fresh names. `instantiate` skips the round trip and substitutes a value straight into the closed body, which is exactly beta reduction.

```rust
#![function!("unbound/src/lib.rs", normalize)]
```

The classic capture hazard is `(\x. \y. x) y`. Naive substitution produces `\y. y`, the identity. Here the inner `y` is a de Bruijn index and the free `y` is a name, so they cannot collide and the result is the constant function returning the free `y`.

## Printing

Opening a binder yields a fresh index but may reuse a spelling, so printing names naively can show `\y. y` for a term whose body is the free `y`. `NameScope` tracks the spelling of every name in scope and renames a binder only when it would capture a name the body actually uses.

```rust
#![function!("unbound/src/lib.rs", go)]
```

The capture example above prints as `\y1. y`, which reads back as the same term.

## Best Practices

Reach into a binder through `unbind` or `unbind_ref`, never `body`. The body is stored closed and its bound variables are raw de Bruijn coordinates.

Prefer `instantiate` for beta reduction and type application. It avoids generating fresh names that are immediately substituted away.

Use `Vec<Name<T>>` as the pattern for binders that introduce several names at once, such as `forall a b. t`, and `(Name<T>, Ann)` for annotated binders. An annotation sits outside the scope of its own binder.

Use `#[subst(Ty)]` on a term type to derive substitution of types into terms, which is what type application in System F needs. Use `#[subst(_)]` for types with no variables of their own.

Freshness comes from a global counter, so `unbind` is safe anywhere. Reach for `FreshM` only when you want readable fresh spellings like `x1` in output.
