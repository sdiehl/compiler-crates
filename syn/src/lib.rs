//! # Syn Examples for Compiler Construction
//!
//! Demonstrates how syn can be used for various compiler-related tasks, from
//! parsing Rust syntax to building custom languages that integrate with Rust.

use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_quote, Error, Expr, ExprLit, FnArg, ItemFn, Lit, Pat, Result, Stmt, Token, Type,
    Visibility,
};

/// Example: Parsing and analyzing a Rust function
pub fn analyze_function(input: TokenStream) -> Result<FunctionAnalysis> {
    let func: ItemFn = syn::parse2(input)?;

    let param_count = func.sig.inputs.len();
    let is_async = func.sig.asyncness.is_some();
    let is_unsafe = func.sig.unsafety.is_some();
    let has_generics = !func.sig.generics.params.is_empty();

    let params = func
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(ident) = pat_type.pat.as_ref() {
                    Some(ident.ident.to_string())
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();

    Ok(FunctionAnalysis {
        name: func.sig.ident.to_string(),
        param_count,
        params,
        is_async,
        is_unsafe,
        has_generics,
        visibility: format!("{:?}", func.vis),
    })
}

#[derive(Debug, Clone)]
pub struct FunctionAnalysis {
    pub name: String,
    pub param_count: usize,
    pub params: Vec<String>,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub has_generics: bool,
    pub visibility: String,
}

/// Example: Custom DSL parsing - Simple state machine language
pub struct StateMachine {
    pub name: Ident,
    pub states: Vec<State>,
    pub initial: Ident,
}

pub struct State {
    pub name: Ident,
    pub transitions: Vec<Transition>,
}

pub struct Transition {
    pub event: Ident,
    pub target: Ident,
    pub action: Option<Expr>,
}

impl Parse for StateMachine {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::state>()?;
        input.parse::<kw::machine>()?;
        let name: Ident = input.parse()?;

        let content;
        syn::braced!(content in input);

        // Parse initial state
        content.parse::<kw::initial>()?;
        content.parse::<Token![:]>()?;
        let initial: Ident = content.parse()?;
        content.parse::<Token![;]>()?;

        // Parse states
        let mut states = Vec::new();
        while !content.is_empty() {
            states.push(content.parse()?);
        }

        Ok(StateMachine {
            name,
            states,
            initial,
        })
    }
}

impl Parse for State {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::state>()?;
        let name: Ident = input.parse()?;

        let content;
        syn::braced!(content in input);

        let mut transitions = Vec::new();
        while !content.is_empty() {
            transitions.push(content.parse()?);
        }

        Ok(State { name, transitions })
    }
}

impl Parse for Transition {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::on>()?;
        let event: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let target: Ident = input.parse()?;

        let action = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        input.parse::<Token![;]>()?;
        Ok(Transition {
            event,
            target,
            action,
        })
    }
}

// Custom keywords for our DSL
mod kw {
    use syn::custom_keyword;

    custom_keyword!(state);
    custom_keyword!(machine);
    custom_keyword!(initial);
    custom_keyword!(on);
}

/// Example: AST transformation - Add logging to functions
pub fn inject_logging(mut func: ItemFn) -> ItemFn {
    let fn_name = &func.sig.ident;
    let log_entry: Stmt = parse_quote! {
        println!("Entering function: {}", stringify!(#fn_name));
    };

    // Insert at the beginning of the function body
    func.block.stmts.insert(0, log_entry);

    // Add exit logging before each return
    let log_exit: Stmt = parse_quote! {
        println!("Exiting function: {}", stringify!(#fn_name));
    };

    let mut new_stmts = Vec::new();
    for stmt in func.block.stmts.drain(..) {
        match &stmt {
            Stmt::Expr(Expr::Return(_), _) => {
                new_stmts.push(log_exit.clone());
                new_stmts.push(stmt);
            }
            _ => new_stmts.push(stmt),
        }
    }

    // Add exit log at the end if there's no explicit return
    if !matches!(new_stmts.last(), Some(Stmt::Expr(Expr::Return(_), _))) {
        new_stmts.push(log_exit);
    }

    func.block.stmts = new_stmts;
    func
}

/// Example: Custom attribute parsing
#[derive(Debug)]
pub struct CompilerDirective {
    pub optimization_level: u8,
    pub inline: bool,
    pub target_features: Vec<String>,
}

impl Parse for CompilerDirective {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut optimization_level = 0;
        let mut inline = false;
        let mut target_features = Vec::new();

        let vars = Punctuated::<MetaItem, Token![,]>::parse_terminated(input)?;
        for var in vars {
            match var.name.to_string().as_str() {
                "opt_level" => optimization_level = var.value,
                "inline" => inline = true,
                "features" => {
                    target_features = var
                        .list
                        .into_iter()
                        .map(|s| s.trim_matches('"').to_string())
                        .collect();
                }
                _ => {
                    return Err(Error::new(
                        var.name.span(),
                        format!("Unknown directive: {}", var.name),
                    ))
                }
            }
        }

        Ok(CompilerDirective {
            optimization_level,
            inline,
            target_features,
        })
    }
}

struct MetaItem {
    name: Ident,
    value: u8,
    list: Vec<String>,
}

impl Parse for MetaItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;

        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            if let Ok(lit) = input.parse::<ExprLit>() {
                if let Lit::Int(int) = lit.lit {
                    let value = int.base10_parse::<u8>()?;
                    return Ok(MetaItem {
                        name,
                        value,
                        list: vec![],
                    });
                }
            }
        }

        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let list = Punctuated::<ExprLit, Token![,]>::parse_terminated(&content)?
                .into_iter()
                .filter_map(|lit| {
                    if let Lit::Str(s) = lit.lit {
                        Some(s.value())
                    } else {
                        None
                    }
                })
                .collect();
            return Ok(MetaItem {
                name,
                value: 0,
                list,
            });
        }

        Ok(MetaItem {
            name,
            value: 1,
            list: vec![],
        })
    }
}

/// Example: Type analysis for compiler optimizations
pub fn analyze_types_in_function(func: &ItemFn) -> HashMap<String, TypeInfo> {
    let mut type_info = HashMap::new();

    // Analyze parameter types
    for input in &func.sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            if let Pat::Ident(ident) = pat_type.pat.as_ref() {
                let info = analyze_type(&pat_type.ty);
                type_info.insert(ident.ident.to_string(), info);
            }
        }
    }

    type_info
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub is_primitive: bool,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub type_string: String,
}

fn analyze_type(ty: &Type) -> TypeInfo {
    match ty {
        Type::Path(type_path) => {
            let type_string = quote!(#type_path).to_string();
            let is_primitive = matches!(
                type_string.as_str(),
                "i8" | "i16"
                    | "i32"
                    | "i64"
                    | "i128"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "f32"
                    | "f64"
                    | "bool"
                    | "char"
            );
            TypeInfo {
                is_primitive,
                is_reference: false,
                is_mutable: false,
                type_string,
            }
        }
        Type::Reference(type_ref) => {
            let inner = analyze_type(&type_ref.elem);
            TypeInfo {
                is_reference: true,
                is_mutable: type_ref.mutability.is_some(),
                ..inner
            }
        }
        _ => TypeInfo {
            is_primitive: false,
            is_reference: false,
            is_mutable: false,
            type_string: quote!(#ty).to_string(),
        },
    }
}

/// Example: Generate optimized code based on const evaluation
pub fn const_fold_binary_ops(expr: Expr) -> Expr {
    match expr {
        Expr::Binary(mut binary) => {
            // Recursively fold sub-expressions
            binary.left = Box::new(const_fold_binary_ops(*binary.left));
            binary.right = Box::new(const_fold_binary_ops(*binary.right));

            // Try to fold if both operands are literals
            if let (Expr::Lit(left_lit), Expr::Lit(right_lit)) =
                (binary.left.as_ref(), binary.right.as_ref())
            {
                if let (Lit::Int(l), Lit::Int(r)) = (&left_lit.lit, &right_lit.lit) {
                    if let (Ok(l_val), Ok(r_val)) =
                        (l.base10_parse::<i64>(), r.base10_parse::<i64>())
                    {
                        use syn::BinOp;
                        let result = match binary.op {
                            BinOp::Add(_) => Some(l_val + r_val),
                            BinOp::Sub(_) => Some(l_val - r_val),
                            BinOp::Mul(_) => Some(l_val * r_val),
                            BinOp::Div(_) if r_val != 0 => Some(l_val / r_val),
                            _ => None,
                        };

                        if let Some(val) = result {
                            return parse_quote!(#val);
                        }
                    }
                }
            }

            Expr::Binary(binary)
        }
        // Recursively process other expression types
        Expr::Paren(mut paren) => {
            paren.expr = Box::new(const_fold_binary_ops(*paren.expr));
            Expr::Paren(paren)
        }
        Expr::Block(mut block) => {
            if let Some(Stmt::Expr(expr, _semi)) = block.block.stmts.last_mut() {
                *expr = const_fold_binary_ops(expr.clone());
            }
            Expr::Block(block)
        }
        other => other,
    }
}

/// Error handling with span information
pub fn validate_function(func: &ItemFn) -> std::result::Result<(), Vec<Error>> {
    let mut errors = Vec::new();

    // Check function name conventions
    let name = func.sig.ident.to_string();
    if name.starts_with('_') && func.vis != Visibility::Inherited {
        errors.push(Error::new(
            func.sig.ident.span(),
            "Public functions should not start with underscore",
        ));
    }

    // Check for missing documentation
    if !func.attrs.iter().any(|attr| attr.path().is_ident("doc")) {
        errors.push(Error::new(
            func.sig.ident.span(),
            "Missing documentation comment",
        ));
    }

    // Check parameter conventions
    for input in &func.sig.inputs {
        let FnArg::Typed(pat_type) = input else {
            continue;
        };

        let Type::Reference(type_ref) = pat_type.ty.as_ref() else {
            continue;
        };

        if type_ref.mutability.is_some() {
            continue;
        }

        let Type::Path(path) = type_ref.elem.as_ref() else {
            continue;
        };

        let Some(ident) = path.path.get_ident() else {
            continue;
        };

        let type_name = ident.to_string();
        if matches!(type_name.as_str(), "String" | "Vec" | "HashMap") {
            errors.push(Error::new(
                pat_type.ty.span(),
                format!(
                    "Consider using &{} instead of {} for better performance",
                    type_name, type_name
                ),
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_analysis() {
        let input = quote! {
            pub async unsafe fn process_data<T>(input: &str, count: usize) -> Result<T> {
                todo!()
            }
        };

        let analysis = analyze_function(input).unwrap();
        assert_eq!(analysis.name, "process_data");
        assert_eq!(analysis.param_count, 2);
        assert!(analysis.is_async);
        assert!(analysis.is_unsafe);
        assert!(analysis.has_generics);
        assert_eq!(analysis.params, vec!["input", "count"]);
    }

    #[test]
    fn test_inject_logging() {
        let input: ItemFn = parse_quote! {
            fn calculate(x: i32, y: i32) -> i32 {
                if x > y {
                    return x - y;
                }
                x + y
            }
        };

        let modified = inject_logging(input);
        let output = quote!(#modified).to_string();
        assert!(output.contains("Entering function"));
        assert!(output.contains("Exiting function"));
    }

    #[test]
    fn test_const_folding() {
        // Test simple constant folding
        let expr: Expr = parse_quote! { 2 + 3 };
        let folded = const_fold_binary_ops(expr);
        match &folded {
            Expr::Lit(lit) => {
                if let Lit::Int(int) = &lit.lit {
                    assert_eq!(int.base10_parse::<i64>().unwrap(), 5);
                } else {
                    panic!("Expected integer literal");
                }
            }
            _ => panic!(
                "Expected literal after folding, got: {:?}",
                quote!(#folded).to_string()
            ),
        }

        // Test division
        let expr: Expr = parse_quote! { 10 / 2 };
        let folded = const_fold_binary_ops(expr);
        if let Expr::Lit(lit) = &folded {
            if let Lit::Int(int) = &lit.lit {
                assert_eq!(int.base10_parse::<i64>().unwrap(), 5);
            }
        }

        // Test non-foldable expression (variable)
        let expr: Expr = parse_quote! { x + 3 };
        let folded = const_fold_binary_ops(expr);
        assert!(matches!(folded, Expr::Binary(_)));
    }

    #[test]
    fn test_type_analysis() {
        let func: ItemFn = parse_quote! {
            fn example(x: i32, s: &str, data: &mut Vec<u8>) {}
        };

        let types = analyze_types_in_function(&func);
        assert!(types["x"].is_primitive);
        assert!(types["s"].is_reference);
        assert!(!types["s"].is_mutable);
        assert!(types["data"].is_reference);
        assert!(types["data"].is_mutable);
    }
}
