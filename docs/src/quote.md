# quote

quote provides quasi-quoting for generating Rust code programmatically while preserving the structure and formatting of the generated code. The library works in tandem with syn for parsing and proc-macro2 for token manipulation, forming the foundation of most Rust procedural macros. Unlike string-based code generation, quote maintains type safety and proper hygiene while generating syntactically correct Rust code.

The library's interpolation syntax using # allows embedding runtime values into generated code, while repetition patterns with #(...)* enable generating loops and repeated structures. quote excels at preserving the visual structure of code templates, making generated code readable and maintainable. The ability to splice together token streams from different sources enables modular code generation patterns.

## Basic Code Generation

```rust
use quote::{quote, format_ident};
use proc_macro2::TokenStream;

pub fn generate_function(name: &str, body: TokenStream) -> TokenStream {
    let fn_name = format_ident!("{}", name);

    quote! {
        pub fn #fn_name() -> i32 {
            #body
        }
    }
}

pub fn generate_struct(name: &str, fields: &[(String, String)]) -> TokenStream {
    let struct_name = format_ident!("{}", name);
    let field_defs = fields.iter().map(|(name, ty)| {
        let field_name = format_ident!("{}", name);
        let field_type = format_ident!("{}", ty);
        quote! { pub #field_name: #field_type }
    });

    quote! {
        #[derive(Debug, Clone)]
        pub struct #struct_name {
            #(#field_defs),*
        }
    }
}
```

The basic code generation functions demonstrate quote's fundamental interpolation mechanism. The format_ident! macro creates identifiers from strings, ensuring they are valid Rust identifiers. The # symbol acts as an interpolation marker, embedding the identifier into the generated code. The quote! macro preserves the visual structure of the code template, making it easy to understand what code will be generated.

The repetition pattern #(#field_defs),* generates a comma-separated list of field definitions. This pattern iterates over the field_defs iterator, inserting each element separated by commas. The outer #(...) marks the repetition boundary, the inner # interpolates each item, and the ,* specifies comma separation with zero or more repetitions.

## AST-Based Generation

```rust
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(i32),
    Variable(String),
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add, Sub, Mul, Div,
}

impl quote::ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Expr::Literal(n) => tokens.extend(quote! { #n }),
            Expr::Variable(name) => {
                let ident = format_ident!("{}", name);
                tokens.extend(quote! { #ident });
            }
            Expr::Binary { op, left, right } => {
                let op_tokens = match op {
                    BinaryOp::Add => quote! { + },
                    BinaryOp::Sub => quote! { - },
                    BinaryOp::Mul => quote! { * },
                    BinaryOp::Div => quote! { / },
                };
                tokens.extend(quote! {
                    (#left #op_tokens #right)
                });
            }
        }
    }
}
```

Implementing ToTokens allows custom types to participate in quote's interpolation system. The to_tokens method converts the AST representation into token streams that represent valid Rust code. This approach enables type-safe code generation where the AST structure ensures only valid combinations are possible.

The recursive nature of the Binary variant demonstrates how complex expressions naturally map to nested token generation. The parentheses in the output ensure proper precedence, while the interpolation of left and right recursively invokes their ToTokens implementations. This pattern scales to arbitrarily complex ASTs while maintaining clean separation between representation and generation.

## Builder Pattern Generation

```rust
pub fn generate_builder(struct_name: &str, fields: &[(String, String)]) -> TokenStream {
    let struct_ident = format_ident!("{}", struct_name);
    let builder_ident = format_ident!("{}Builder", struct_name);

    let builder_fields = fields.iter().map(|(name, ty)| {
        let name = format_ident!("{}", name);
        let ty = format_ident!("{}", ty);
        quote! { #name: Option<#ty> }
    });

    let builder_methods = fields.iter().map(|(name, ty)| {
        let name = format_ident!("{}", name);
        let ty = format_ident!("{}", ty);
        quote! {
            pub fn #name(mut self, value: #ty) -> Self {
                self.#name = Some(value);
                self
            }
        }
    });

    let build_assignments = fields.iter().map(|(name, _)| {
        let name = format_ident!("{}", name);
        let error_msg = format!("Field {} is required", name);
        quote! {
            #name: self.#name.ok_or(#error_msg)?
        }
    });

    quote! {
        pub struct #builder_ident {
            #(#builder_fields),*
        }

        impl #builder_ident {
            pub fn new() -> Self {
                Self {
                    #(#name: None),*
                }
            }

            #(#builder_methods)*

            pub fn build(self) -> Result<#struct_ident, &'static str> {
                Ok(#struct_ident {
                    #(#build_assignments),*
                })
            }
        }
    }
}
```

The builder pattern generator showcases quote's ability to generate complex patterns with multiple related components. Each field generates three pieces: an optional field in the builder, a setter method, and a build-time assignment. The repetition patterns handle collections of generated code, maintaining consistency across all fields.

The error handling in the build method demonstrates embedding runtime values into generated code. The format! macro creates error messages at generation time, which become string literals in the generated code. This technique allows customizing generated code based on input parameters while maintaining compile-time type checking.

## Trait Implementation Generation

```rust
pub fn generate_display_impl(
    struct_name: &str,
    format_str: &str,
    fields: &[String]
) -> TokenStream {
    let struct_ident = format_ident!("{}", struct_name);
    let field_refs = fields.iter().map(|name| {
        let field = format_ident!("{}", name);
        quote! { self.#field }
    });

    quote! {
        impl std::fmt::Display for #struct_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, #format_str, #(#field_refs),*)
            }
        }
    }
}

pub fn generate_from_impl(
    target: &str,
    source: &str,
    conversion: TokenStream
) -> TokenStream {
    let target_ident = format_ident!("{}", target);
    let source_ident = format_ident!("{}", source);

    quote! {
        impl From<#source_ident> for #target_ident {
            fn from(value: #source_ident) -> Self {
                #conversion
            }
        }
    }
}
```

Trait implementation generation demonstrates quote's ability to generate standard Rust patterns. The Display implementation shows how format strings and field references combine to create formatted output. The repetition pattern in write! generates the exact number of arguments needed, matching the format string placeholders.

The From implementation generator accepts a TokenStream for the conversion logic, showing how quote enables composition of generated code. This pattern allows callers to provide complex conversion logic while the generator handles the boilerplate trait implementation structure.

## Generic Code Generation

```rust
pub fn generate_generic_wrapper<T: quote::ToTokens>(
    name: &str,
    inner_type: T,
    bounds: &[String]
) -> TokenStream {
    let wrapper_ident = format_ident!("{}", name);
    let bound_tokens = bounds.iter().map(|b| {
        let bound = format_ident!("{}", b);
        quote! { #bound }
    });

    quote! {
        pub struct #wrapper_ident<T>
        where
            T: #(#bound_tokens)+*
        {
            inner: #inner_type,
            phantom: std::marker::PhantomData<T>,
        }

        impl<T> #wrapper_ident<T>
        where
            T: #(#bound_tokens)+*
        {
            pub fn new(inner: #inner_type) -> Self {
                Self {
                    inner,
                    phantom: std::marker::PhantomData,
                }
            }

            pub fn into_inner(self) -> #inner_type {
                self.inner
            }
        }
    }
}
```

Generic code generation requires careful handling of type parameters and bounds. The bound_tokens iterator generates trait bounds, while the +* repetition pattern creates the proper + separator for multiple bounds. The where clause repetition ensures bounds appear consistently in both the struct definition and implementation.

The PhantomData field demonstrates generating standard patterns for generic types that don't directly use their type parameters. This pattern is essential for maintaining proper variance and drop checking in generic types.

## Method Chain Generation

```rust
pub fn generate_method_chain(
    base: TokenStream,
    methods: &[(String, Vec<TokenStream>)]
) -> TokenStream {
    let mut result = base;

    for (method, args) in methods {
        let method_ident = format_ident!("{}", method);
        result = quote! {
            #result.#method_ident(#(#args),*)
        };
    }

    result
}

pub fn generate_builder_chain(fields: &[(String, TokenStream)]) -> TokenStream {
    let setters = fields.iter().map(|(name, value)| {
        let method = format_ident!("{}", name);
        quote! { .#method(#value) }
    });

    quote! {
        Builder::new()
            #(#setters)*
            .build()
    }
}
```

Method chain generation shows how quote enables building complex expressions programmatically. The iterative approach accumulates method calls, with each iteration wrapping the previous result. This pattern generates fluent interfaces and builder chains commonly used in Rust APIs.

The builder chain generator demonstrates generating idiomatic Rust patterns with proper indentation and formatting. The quote! macro preserves the visual structure, making the generated code readable. The repetition pattern handles any number of setter calls while maintaining consistent formatting.

## Conditional Generation

```rust
pub fn generate_conditional_impl(
    condition: bool,
    true_branch: TokenStream,
    false_branch: TokenStream
) -> TokenStream {
    if condition {
        quote! {
            #[cfg(feature = "enabled")]
            #true_branch
        }
    } else {
        quote! {
            #[cfg(not(feature = "enabled"))]
            #false_branch
        }
    }
}

pub fn generate_optional_field(
    name: &str,
    ty: &str,
    include: bool
) -> TokenStream {
    let field_name = format_ident!("{}", name);
    let field_type = format_ident!("{}", ty);

    if include {
        quote! {
            pub #field_name: #field_type,
        }
    } else {
        quote! {}
    }
}
```

Conditional generation enables creating different code based on compile-time conditions. The cfg attributes in generated code allow feature-gated implementations, while the generation-time conditions customize what code gets generated. This dual-layer approach provides maximum flexibility in code generation.

Empty token streams from quote! {} allow optional elements in generated code. This pattern is useful for conditionally including fields, methods, or entire implementations based on configuration or feature flags.

## Span Preservation

```rust
use quote::quote_spanned;
use proc_macro2::Span;

pub fn generate_spanned_error(span: Span, message: &str) -> TokenStream {
    quote_spanned! {span=>
        compile_error!(#message);
    }
}

pub fn generate_with_location(span: Span, code: TokenStream) -> TokenStream {
    quote_spanned! {span=>
        #code
    }
}
```

The quote_spanned! macro preserves source location information, crucial for error reporting in procedural macros. When the generated code contains errors, the compiler reports them at the original source location rather than pointing to the macro invocation. This feature significantly improves the debugging experience for macro users.

Span preservation enables generating helpful error messages that point to the exact location of problems in the input code. This capability is essential for creating user-friendly procedural macros that provide clear diagnostics.

## Repetition Patterns

```rust
pub fn generate_match_arms(variants: &[(String, TokenStream)]) -> TokenStream {
    let arms = variants.iter().map(|(pattern, body)| {
        let pattern_ident = format_ident!("{}", pattern);
        quote! {
            Self::#pattern_ident => { #body }
        }
    });

    quote! {
        match self {
            #(#arms),*
        }
    }
}

pub fn generate_tuple_destructure(count: usize) -> TokenStream {
    let vars = (0..count).map(|i| format_ident!("_{}", i));
    let indices = (0..count).map(|i| syn::Index::from(i));

    quote! {
        let (#(#vars),*) = tuple;
        #(
            println!("Element {}: {:?}", #indices, #vars);
        )*
    }
}
```

Advanced repetition patterns demonstrate quote's flexibility in generating complex structures. The match arm generation shows how patterns and bodies can be generated from data, while maintaining proper syntax. The comma separator in the repetition ensures valid match syntax.

The tuple destructuring example showcases numeric repetition, generating unique variable names and accessing tuple elements by index. The nested repetition pattern generates both the destructuring and the println statements, demonstrating how multiple repetitions can work together.

## Integration with syn

```rust
use syn::{parse_quote, Expr, Stmt};

pub fn generate_assertion(left: &str, op: &str, right: &str) -> TokenStream {
    let assertion: Expr = parse_quote! {
        assert!(#left #op #right, "Assertion failed: {} {} {}", #left, #op, #right)
    };

    quote! { #assertion }
}

pub fn generate_test_function(name: &str, body: Vec<Stmt>) -> TokenStream {
    let test_name = format_ident!("test_{}", name);

    quote! {
        #[test]
        fn #test_name() {
            #(#body)*
        }
    }
}
```

The parse_quote! macro from syn parses string literals into syn types at compile time, which can then be interpolated with quote!. This combination enables parsing complex expressions and statements while maintaining type safety. The assertion generator shows how string representations convert to properly typed AST nodes.

Integration with syn enables sophisticated code generation patterns where parsing and generation work together. This approach is particularly useful in procedural macros that need to analyze input code before generating output.

## Best Practices

Structure generated code to match handwritten Rust conventions. Use proper indentation and formatting in quote! templates to make generated code readable. The visual structure of the template should reflect the structure of the generated code, making it easy to understand what will be generated.

Implement ToTokens for custom types that frequently appear in generated code. This approach provides better abstraction than repeatedly using quote! for the same patterns. Custom ToTokens implementations can encapsulate complex generation logic while providing a clean interface.

Use format_ident! for creating identifiers from strings, ensuring valid Rust identifiers. Never concatenate strings to build code; use quote!'s interpolation system instead. This approach maintains hygiene and prevents syntax errors in generated code.

Preserve spans when generating error messages or warnings. Use quote_spanned! to attach generated code to specific source locations, improving error messages. Good span preservation makes procedural macros feel like built-in language features.

Test generated code by comparing TokenStream representations or by compiling and running the generated code. Use snapshot testing for complex generated structures, capturing the generated code as strings for comparison. Regular testing ensures code generation remains correct as requirements evolve.

quote provides an elegant and powerful system for generating Rust code that maintains the language's safety guarantees while enabling sophisticated metaprogramming patterns. Its integration with the procedural macro ecosystem makes it indispensable for creating derive macros, attribute macros, and code generators that feel native to Rust.
