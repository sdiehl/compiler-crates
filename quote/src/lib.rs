use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};

// AST Generation Examples

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let { name: String, value: Expr },
    Return(Expr),
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Variable(String),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        func: String,
        args: Vec<Expr>,
    },
    Block(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
}

impl ToTokens for Function {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.name);
        let params = self.params.iter().map(|p| {
            let param_name = format_ident!("{}", p.name);
            let param_type = format_ident!("{}", p.ty);
            quote! { #param_name: #param_type }
        });
        let return_type = format_ident!("{}", self.return_type);
        let body = &self.body;

        tokens.extend(quote! {
            pub fn #name(#(#params),*) -> #return_type {
                #(#body)*
            }
        });
    }
}

impl ToTokens for Statement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Statement::Let { name, value } => {
                let name = format_ident!("{}", name);
                tokens.extend(quote! {
                    let #name = #value;
                });
            }
            Statement::Return(expr) => {
                tokens.extend(quote! {
                    return #expr;
                });
            }
            Statement::Expression(expr) => {
                tokens.extend(quote! {
                    #expr;
                });
            }
        }
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Expr::Literal(lit) => lit.to_tokens(tokens),
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
                    BinaryOp::Eq => quote! { == },
                    BinaryOp::Lt => quote! { < },
                    BinaryOp::Gt => quote! { > },
                };
                tokens.extend(quote! {
                    (#left #op_tokens #right)
                });
            }
            Expr::Call { func, args } => {
                let func = format_ident!("{}", func);
                tokens.extend(quote! {
                    #func(#(#args),*)
                });
            }
            Expr::Block(stmts) => {
                tokens.extend(quote! {
                    {
                        #(#stmts)*
                    }
                });
            }
        }
    }
}

impl ToTokens for Literal {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Literal::Int(n) => tokens.extend(quote! { #n }),
            Literal::Float(f) => tokens.extend(quote! { #f }),
            Literal::String(s) => tokens.extend(quote! { #s }),
            Literal::Bool(b) => tokens.extend(quote! { #b }),
        }
    }
}

// Builder Pattern Generation

pub fn generate_builder(struct_name: &str, fields: &[(String, String)]) -> TokenStream {
    let struct_ident = format_ident!("{}", struct_name);
    let builder_ident = format_ident!("{}Builder", struct_name);

    let field_defs = fields.iter().map(|(name, ty)| {
        let name = format_ident!("{}", name);
        let ty = format_ident!("{}", ty);
        quote! { pub #name: #ty }
    });

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

    let build_field_assigns = fields.iter().map(|(name, _)| {
        let name = format_ident!("{}", name);
        let error_msg = format!("Field {} is required", name);
        quote! {
            #name: self.#name.ok_or(#error_msg)?
        }
    });

    let default_fields = fields.iter().map(|(name, _)| {
        let name = format_ident!("{}", name);
        quote! { #name: None }
    });

    quote! {
        #[derive(Debug, Clone)]
        pub struct #struct_ident {
            #(#field_defs),*
        }

        impl #struct_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident::new()
            }
        }

        pub struct #builder_ident {
            #(#builder_fields),*
        }

        impl #builder_ident {
            pub fn new() -> Self {
                Self {
                    #(#default_fields),*
                }
            }

            #(#builder_methods)*

            pub fn build(self) -> Result<#struct_ident, &'static str> {
                Ok(#struct_ident {
                    #(#build_field_assigns),*
                })
            }
        }
    }
}

// Enum Variant Generation

pub fn generate_enum_matcher(enum_name: &str, variants: &[String]) -> TokenStream {
    let enum_ident = format_ident!("{}", enum_name);
    let match_arms = variants.iter().map(|variant| {
        let variant_ident = format_ident!("{}", variant);
        let variant_str = variant.to_lowercase();
        quote! {
            #enum_ident::#variant_ident => #variant_str
        }
    });

    quote! {
        impl #enum_ident {
            pub fn as_str(&self) -> &'static str {
                match self {
                    #(#match_arms),*
                }
            }
        }
    }
}

// Trait Implementation Generation

pub fn generate_display_impl(
    struct_name: &str,
    format_str: &str,
    field_names: &[String],
) -> TokenStream {
    let struct_ident = format_ident!("{}", struct_name);
    let field_refs = field_names.iter().map(|name| {
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

// Repetition and Interpolation

pub fn generate_vector_wrapper(item_type: &str, methods: &[(&str, &str)]) -> TokenStream {
    let item_ident = format_ident!("{}", item_type);
    let wrapper_ident = format_ident!("{}Vec", item_type);

    let method_impls = methods.iter().map(|(name, return_type)| {
        let method_name = format_ident!("{}", name);
        let return_type = format_ident!("{}", return_type);
        quote! {
            pub fn #method_name(&self) -> #return_type {
                self.items.#method_name()
            }
        }
    });

    quote! {
        pub struct #wrapper_ident {
            items: Vec<#item_ident>,
        }

        impl #wrapper_ident {
            pub fn new() -> Self {
                Self { items: Vec::new() }
            }

            pub fn push(&mut self, item: #item_ident) {
                self.items.push(item);
            }

            #(#method_impls)*
        }
    }
}

// Macro Pattern Generation

pub fn generate_macro_rules(macro_name: &str, patterns: &[(String, String)]) -> TokenStream {
    let macro_ident = format_ident!("{}", macro_name);

    let rules = patterns.iter().map(|(pattern, expansion)| {
        // This is a simplified example - real macro patterns would need proper parsing
        quote! {
            (#pattern) => {
                #expansion
            }
        }
    });

    quote! {
        macro_rules! #macro_ident {
            #(#rules);*
        }
    }
}

// Conditional Code Generation

pub fn generate_conditional_impl(
    ty: &str,
    condition: bool,
    true_impl: TokenStream,
    false_impl: TokenStream,
) -> TokenStream {
    let ty_ident = format_ident!("{}", ty);

    if condition {
        quote! {
            impl #ty_ident {
                #true_impl
            }
        }
    } else {
        quote! {
            impl #ty_ident {
                #false_impl
            }
        }
    }
}

// Loop Unrolling Example

pub fn generate_unrolled_loop(count: usize, body_template: &str) -> TokenStream {
    let iterations = (0..count).map(|i| {
        let index = proc_macro2::Literal::usize_unsuffixed(i);
        // In real use, body_template would be parsed and interpolated properly
        quote! {
            {
                let i = #index;
                println!(#body_template, i);
            }
        }
    });

    quote! {
        #(#iterations)*
    }
}

// Generic Type Generation

pub fn generate_generic_struct(
    name: &str,
    type_params: &[String],
    fields: &[(String, String)],
) -> TokenStream {
    let struct_ident = format_ident!("{}", name);
    let type_params = type_params.iter().map(|p| format_ident!("{}", p));

    let field_defs = fields.iter().map(|(name, ty)| {
        let field_name = format_ident!("{}", name);
        let field_type = format_ident!("{}", ty);
        quote! {
            pub #field_name: #field_type
        }
    });

    quote! {
        pub struct #struct_ident<#(#type_params),*> {
            #(#field_defs),*
        }
    }
}

// Span-based Error Reporting

pub fn generate_spanned_error(span: Span, message: &str) -> TokenStream {
    quote_spanned! {span=>
        compile_error!(#message);
    }
}

// Method Chain Generation

pub fn generate_method_chain(base: &str, methods: &[(&str, Vec<&str>)]) -> TokenStream {
    let mut result = quote! { #base };

    for (method, args) in methods {
        let method_ident = format_ident!("{}", method);
        let arg_tokens = args.iter().map(|arg| {
            let arg_ident = format_ident!("{}", arg);
            quote! { #arg_ident }
        });
        result = quote! {
            #result.#method_ident(#(#arg_tokens),*)
        };
    }

    result
}

// Attribute Generation

pub fn generate_derives(derives: &[&str]) -> TokenStream {
    let derive_idents = derives.iter().map(|d| format_ident!("{}", d));
    quote! {
        #[derive(#(#derive_idents),*)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_generation() {
        let func = Function {
            name: "add".to_string(),
            params: vec![
                Parameter {
                    name: "a".to_string(),
                    ty: "i32".to_string(),
                },
                Parameter {
                    name: "b".to_string(),
                    ty: "i32".to_string(),
                },
            ],
            return_type: "i32".to_string(),
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable("a".to_string())),
                right: Box::new(Expr::Variable("b".to_string())),
            })],
        };

        let tokens = quote! { #func };
        let expected = quote! {
            pub fn add(a: i32, b: i32) -> i32 {
                return (a + b);
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_builder_generation() {
        let fields = vec![
            ("name".to_string(), "String".to_string()),
            ("age".to_string(), "u32".to_string()),
        ];

        let tokens = generate_builder("Person", &fields);
        let output = tokens.to_string();
        assert!(output.contains("PersonBuilder"));
        // Check for function signatures with flexible whitespace
        assert!(output.contains("fn name"));
        assert!(output.contains("mut self"));
        assert!(output.contains("value : String"));
        assert!(output.contains("fn age"));
        assert!(output.contains("value : u32"));
    }

    #[test]
    fn test_enum_matcher() {
        let variants = vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()];
        let tokens = generate_enum_matcher("Color", &variants);
        let output = tokens.to_string();

        // Check with flexible whitespace
        assert!(output.contains("Color :: Red"));
        assert!(output.contains("\"red\""));
        assert!(output.contains("Color :: Green"));
        assert!(output.contains("\"green\""));
        assert!(output.contains("Color :: Blue"));
        assert!(output.contains("\"blue\""));
    }

    #[test]
    fn test_display_impl() {
        let tokens = generate_display_impl(
            "Point",
            "Point({}, {})",
            &["x".to_string(), "y".to_string()],
        );
        let output = tokens.to_string();

        // Check with flexible whitespace
        assert!(output.contains("impl"));
        assert!(output.contains("std"));
        assert!(output.contains("fmt"));
        assert!(output.contains("Display"));
        assert!(output.contains("for Point"));
        assert!(output.contains("self.x") || output.contains("self . x"));
        assert!(output.contains("self.y") || output.contains("self . y"));
    }

    #[test]
    fn test_generic_struct() {
        let type_params = vec!["T".to_string(), "U".to_string()];
        let fields = vec![
            ("first".to_string(), "T".to_string()),
            ("second".to_string(), "U".to_string()),
        ];

        let tokens = generate_generic_struct("Pair", &type_params, &fields);
        assert!(tokens.to_string().contains("struct Pair < T , U >"));
        assert!(tokens.to_string().contains("first : T"));
        assert!(tokens.to_string().contains("second : U"));
    }

    #[test]
    fn test_unrolled_loop() {
        let tokens = generate_unrolled_loop(3, "Iteration {}");
        let output = tokens.to_string();

        assert!(output.contains("let i = 0"));
        assert!(output.contains("let i = 1"));
        assert!(output.contains("let i = 2"));
    }

    #[test]
    fn test_derives() {
        let tokens = generate_derives(&["Debug", "Clone", "PartialEq"]);
        assert_eq!(tokens.to_string(), "# [derive (Debug , Clone , PartialEq)]");
    }

    #[test]
    fn test_expression_generation() {
        let expr = Expr::Call {
            func: "println".to_string(),
            args: vec![
                Expr::Literal(Literal::String("Hello, {}!".to_string())),
                Expr::Variable("name".to_string()),
            ],
        };

        let tokens = quote! { #expr };
        assert!(tokens.to_string().contains("println"));
        assert!(tokens.to_string().contains("\"Hello, {}!\""));
        assert!(tokens.to_string().contains("name"));
    }
}
