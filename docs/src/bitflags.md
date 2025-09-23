# bitflags

The `bitflags` crate provides a macro for generating type-safe bitmask structures. In compiler development, bitflags are essential for efficiently representing sets of boolean options or attributes that can be combined. Common use cases include representing file permissions, compiler optimization flags, access modifiers in programming languages, or node attributes in abstract syntax trees.

The primary advantage of `bitflags` over manual bit manipulation is type safety. Instead of working with raw integer constants and bitwise operations that can lead to errors, `bitflags` generates strongly-typed structures that prevent invalid flag combinations at compile time.

The type safety provided by `bitflags` becomes particularly valuable in large compiler codebases where flags may be passed through multiple layers of abstraction. The compiler ensures you cannot accidentally mix incompatible flag types or use undefined flag values.

## Basic Usage

The `bitflags!` macro generates a struct that wraps an integer type and provides methods for safely manipulating sets of flags:

```rust
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FilePermissions: u32 {
        const READ = 0b0000_0001;
        const WRITE = 0b0000_0010;
        const EXECUTE = 0b0000_0100;
        const READ_WRITE = Self::READ.bits() | Self::WRITE.bits();
        const ALL = Self::READ.bits() | Self::WRITE.bits() | Self::EXECUTE.bits();
    }
}
```

This generates a struct with associated constants for each flag. The macro automatically implements common traits like `Debug`, `Clone`, and comparison operators. Each flag is assigned a bit position, and you can define composite flags that combine multiple bits.

## Compiler Flags Example

A practical example in compiler development is managing compiler flags that control various aspects of the compilation process:

```rust
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CompilerFlags: u32 {
        const OPTIMIZE = 1 << 0;
        const DEBUG_INFO = 1 << 1;
        const WARNINGS_AS_ERRORS = 1 << 2;
        const VERBOSE = 1 << 3;
        const LINK_TIME_OPTIMIZATION = 1 << 4;
        const STATIC_LINKING = 1 << 5;
        const PROFILE = 1 << 6;
        const RELEASE = Self::OPTIMIZE.bits() | Self::LINK_TIME_OPTIMIZATION.bits();
        const DEBUG = Self::DEBUG_INFO.bits() | Self::VERBOSE.bits();
    }
}
```

Note how we define composite flags like `RELEASE` and `DEBUG` that combine multiple individual flags. This pattern is common in compilers where certain modes imply specific sets of options.

## Working with Flags

The generated types provide a rich API for manipulating flag sets. You can combine flags using the bitwise OR operator, check if specific flags are set, and perform set operations:

```rust
#![function!("bitflags/src/lib.rs", demonstrate_compiler_flags)]
```

The `contains` method checks if specific flags are set, while `intersection` returns flags common to both sets. Other useful operations include `union` for combining flag sets, `difference` for flags in one set but not another, and `toggle` for flipping specific flags.

## Integration with Compiler Structures

Bitflags integrate naturally with other compiler data structures. Here's an example of using flags within a larger compiler options structure:

```rust
#![struct!("bitflags/src/lib.rs", CompilerOptions)]
```

```rust
#![impl!("bitflags/src/lib.rs", CompilerOptions)]
```

This pattern allows you to encapsulate flag-based configuration with derived state and behavior. The compiler options structure can make decisions based on flag combinations and expose higher-level methods that abstract over the underlying bit manipulation.

## File Permissions Example

Another common use case is representing file permissions or access modifiers:

```rust
#![function!("bitflags/src/lib.rs", demonstrate_file_permissions)]
```

The methods `insert` and `remove` modify flag sets in place, while `intersection` checks for overlapping permissions. This API is much clearer than manual bit manipulation and prevents common errors like using the wrong bit mask.
