# im

The `im` crate provides persistent immutable collections. Updating a map returns a new map that shares almost all of its structure with the old one, so both versions stay valid and a clone costs a pointer copy. `OrdMap` is a B-tree, `HashMap` is a hash array mapped trie, and `Vector` is an RRB tree. All three are designed around cheap `clone`.

This is the natural shape for a typing environment. A mutable symbol table needs a scope stack or an undo log, and every early return from the checker has to remember to pop. With a persistent map a binder builds an extended environment, passes it down, and the caller's map is untouched. Leaving a scope is just returning.

## Typing Environments

The context maps names to types. `OrdMap` iterates in key order, which keeps any diagnostic that prints the environment deterministic.

```rust
#![enum!("im/src/lib.rs", Ty)]
```

```rust
#![enum!("im/src/lib.rs", Expr)]
```

`update` returns a new map with one binding added or replaced. Lambda and let both check their body under `env.update(x, t)` and never mutate `env` itself.

```rust
#![function!("im/src/lib.rs", check)]
```

Checking `(\x: Bool. x) true` under an environment where `x : Int` yields `Bool`, and afterwards the outer `x` is still `Int`. Shadowing needs no special handling.

## REPL History

Because old versions are never destroyed, keeping all of them is nearly free. Each entry in the history shares everything except the path to the changed key with its predecessor.

```rust
#![struct!("im/src/lib.rs", Session)]
```

```rust
#![impl!("im/src/lib.rs", Session)]
```

`undo` pops back to the previous environment with no bookkeeping. `diff` walks only the subtrees the two versions do not share, so reporting what a definition changed is proportional to the change, not the size of the environment.

## Best Practices

Use `OrdMap` when iteration order is observable, as in error messages, generated code or anything hashed into a cache key. Use `im::HashMap` when it is not and lookups dominate.

Pass environments by reference and extend with `update`. Clone only when storing a version, and even then the clone is constant time.

Speculative checking is free. Try an alternative under a cloned environment and discard it on failure. There is nothing to roll back.

Values should be cheap to clone or wrapped in `Rc`. Persistent maps clone values on the paths they copy.

`im` is feature complete and no longer actively developed. The `imbl` crate is a maintained fork with the same API if you need newer fixes.
