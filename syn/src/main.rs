//! Demonstrations of syn for compiler construction tasks

use quote::quote;
use syn::{parse_quote, ItemFn};
use syn_example::{
    analyze_function, analyze_types_in_function, const_fold_binary_ops, inject_logging,
    validate_function, CompilerDirective, StateMachine,
};

fn main() {
    println!("=== Syn Examples for Compiler Construction ===\n");

    // Example 1: Function analysis
    println!("Function Analysis Example:");
    let complex_function = quote! {
        pub async unsafe fn process_data<T, E>(
            input: &str,
            count: usize,
            options: Option<Config>
        ) -> Result<Vec<T>, E>
        where
            T: Clone + Debug,
            E: From<std::io::Error>
        {
            todo!()
        }
    };

    match analyze_function(complex_function) {
        Ok(analysis) => {
            println!("  Function name: {}", analysis.name);
            println!(
                "  Parameters: {} ({})",
                analysis.param_count,
                analysis.params.join(", ")
            );
            println!("  Async: {}", analysis.is_async);
            println!("  Unsafe: {}", analysis.is_unsafe);
            println!("  Generic: {}", analysis.has_generics);
            println!("  Visibility: {}", analysis.visibility);
        }
        Err(e) => eprintln!("  Error: {}", e),
    }

    // Example 2: Logging injection
    println!("\nLogging Injection Example:");
    let original: ItemFn = parse_quote! {
        fn calculate(x: i32, y: i32) -> i32 {
            if x > y {
                return x - y;
            }
            x + y
        }
    };

    println!("Original function:");
    println!("{}", quote!(#original));

    let with_logging = inject_logging(original);
    println!("\nWith logging:");
    println!("{}", quote!(#with_logging));

    // Example 3: Type analysis
    println!("\nType Analysis Example:");
    let typed_function: ItemFn = parse_quote! {
        fn process(
            primitive: u32,
            borrowed: &str,
            mutable_ref: &mut Vec<u8>,
            owned: String
        ) {
            // Implementation
        }
    };

    let type_info = analyze_types_in_function(&typed_function);
    for (param, info) in &type_info {
        println!("  Parameter '{}': {}", param, info.type_string);
        println!("    - Primitive: {}", info.is_primitive);
        println!("    - Reference: {}", info.is_reference);
        println!("    - Mutable: {}", info.is_mutable);
    }

    // Example 4: Constant folding
    println!("\nConstant Folding Example:");
    let expressions = vec![
        quote! { 2 + 3 },
        quote! { 10 - 5 },
        quote! { 4 * 6 },
        quote! { 20 / 4 },
        quote! { (2 + 3) * 4 },
    ];

    for expr_tokens in expressions {
        let expr = syn::parse2(expr_tokens.clone()).unwrap();
        let folded = const_fold_binary_ops(expr);
        println!("  {} => {}", expr_tokens, quote!(#folded));
    }

    // Example 5: Custom DSL parsing
    println!("\nCustom DSL Parsing Example:");
    let state_machine_dsl = quote! {
        state machine TrafficLight {
            initial: Red;

            state Red {
                on timer => Green;
            }

            state Green {
                on timer => Yellow;
            }

            state Yellow {
                on timer => Red;
            }
        }
    };

    println!("Parsing state machine DSL:");
    match syn::parse2::<StateMachine>(state_machine_dsl) {
        Ok(sm) => {
            println!("  State Machine: {}", sm.name);
            println!("  Initial State: {}", sm.initial);
            println!("  States:");
            for state in &sm.states {
                println!("    - {}", state.name);
                for transition in &state.transitions {
                    println!("      {} -> {}", transition.event, transition.target);
                }
            }
        }
        Err(e) => eprintln!("  Parse error: {}", e),
    }

    // Example 6: Function validation
    println!("\nFunction Validation Example:");
    let functions_to_validate: Vec<ItemFn> = vec![
        parse_quote! {
            pub fn _internal_helper() {}
        },
        parse_quote! {
            fn process_data(data: String) -> String {
                data
            }
        },
        parse_quote! {
            /// Documented function
            pub fn good_function(data: &str) -> &str {
                data
            }
        },
    ];

    for (i, func) in functions_to_validate.iter().enumerate() {
        println!("  Function {}: {}", i + 1, func.sig.ident);
        match validate_function(func) {
            Ok(()) => println!("    ✓ Validation passed"),
            Err(errors) => {
                for error in errors {
                    println!("    ✗ {}", error);
                }
            }
        }
    }

    // Example 7: Compiler directive parsing
    println!("\nCompiler Directive Parsing Example:");
    let directive_examples = vec![
        quote! { opt_level = 3, inline, features("sse4", "avx2") },
        quote! { opt_level = 2 },
        quote! { inline, features("neon") },
    ];

    for tokens in directive_examples {
        match syn::parse2::<CompilerDirective>(tokens.clone()) {
            Ok(directive) => {
                println!("  Parsed: {}", tokens);
                println!("    Optimization: O{}", directive.optimization_level);
                println!("    Inline: {}", directive.inline);
                if !directive.target_features.is_empty() {
                    println!("    Features: {}", directive.target_features.join(", "));
                }
            }
            Err(e) => eprintln!("  Error parsing '{}': {}", tokens, e),
        }
    }
}
