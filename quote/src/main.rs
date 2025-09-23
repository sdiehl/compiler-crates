use quote::quote;
use quote_example::{
    generate_builder, generate_conditional_impl, generate_derives, generate_display_impl,
    generate_enum_matcher, generate_generic_struct, generate_method_chain, generate_unrolled_loop,
    generate_vector_wrapper, BinaryOp, Expr, Function, Literal, Parameter, Statement,
};

fn main() {
    println!("=== Quote Code Generation Examples ===\n");

    // Generate a simple function
    println!("Generated Function:");
    let func = Function {
        name: "calculate".to_string(),
        params: vec![
            Parameter {
                name: "x".to_string(),
                ty: "i32".to_string(),
            },
            Parameter {
                name: "y".to_string(),
                ty: "i32".to_string(),
            },
        ],
        return_type: "i32".to_string(),
        body: vec![
            Statement::Let {
                name: "result".to_string(),
                value: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Binary {
                        op: BinaryOp::Mul,
                        left: Box::new(Expr::Variable("x".to_string())),
                        right: Box::new(Expr::Literal(Literal::Int(2))),
                    }),
                    right: Box::new(Expr::Variable("y".to_string())),
                },
            },
            Statement::Return(Expr::Variable("result".to_string())),
        ],
    };

    let func_tokens = quote! { #func };
    println!("{}\n", func_tokens);

    // Generate a builder pattern
    println!("Generated Builder Pattern:");
    let fields = vec![
        ("id".to_string(), "u64".to_string()),
        ("name".to_string(), "String".to_string()),
        ("email".to_string(), "String".to_string()),
        ("active".to_string(), "bool".to_string()),
    ];

    let builder_tokens = generate_builder("User", &fields);
    println!("{}\n", builder_tokens);

    // Generate enum matcher
    println!("Generated Enum Matcher:");
    let variants = vec![
        "Pending".to_string(),
        "Processing".to_string(),
        "Completed".to_string(),
        "Failed".to_string(),
    ];

    let matcher_tokens = generate_enum_matcher("Status", &variants);
    println!("{}\n", matcher_tokens);

    // Generate Display implementation
    println!("Generated Display Implementation:");
    let display_tokens = generate_display_impl(
        "Coordinate",
        "({}, {}, {})",
        &["x".to_string(), "y".to_string(), "z".to_string()],
    );
    println!("{}\n", display_tokens);

    // Generate generic struct
    println!("Generated Generic Struct:");
    let type_params = vec!["K".to_string(), "V".to_string()];
    let generic_fields = vec![
        ("key".to_string(), "K".to_string()),
        ("value".to_string(), "V".to_string()),
        ("timestamp".to_string(), "u64".to_string()),
    ];

    let generic_tokens = generate_generic_struct("Entry", &type_params, &generic_fields);
    println!("{}\n", generic_tokens);

    // Generate vector wrapper
    println!("Generated Vector Wrapper:");
    let methods = vec![("len", "usize"), ("is_empty", "bool")];

    let wrapper_tokens = generate_vector_wrapper("Item", &methods);
    println!("{}\n", wrapper_tokens);

    // Generate unrolled loop
    println!("Generated Unrolled Loop:");
    let loop_tokens = generate_unrolled_loop(4, "Processing item {}");
    println!("{}\n", loop_tokens);

    // Generate conditional implementation
    println!("Generated Conditional Implementation:");
    let true_impl = quote! {
        pub fn enabled_feature(&self) -> bool {
            true
        }
    };

    let false_impl = quote! {
        pub fn disabled_feature(&self) -> bool {
            false
        }
    };

    let conditional_tokens = generate_conditional_impl("Config", true, true_impl, false_impl);
    println!("{}\n", conditional_tokens);

    // Generate derive attributes
    println!("Generated Derive Attributes:");
    let derives = generate_derives(&["Debug", "Clone", "PartialEq", "Serialize", "Deserialize"]);
    println!("{}\n", derives);

    // Complex expression generation
    println!("Generated Complex Expression:");
    let complex_expr = Expr::Block(vec![
        Statement::Let {
            name: "temp".to_string(),
            value: Expr::Call {
                func: "process".to_string(),
                args: vec![Expr::Variable("input".to_string())],
            },
        },
        Statement::Expression(Expr::Call {
            func: "validate".to_string(),
            args: vec![Expr::Variable("temp".to_string())],
        }),
        Statement::Return(Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::Variable("temp".to_string())),
            right: Box::new(Expr::Literal(Literal::Int(0))),
        }),
    ]);

    let expr_tokens = quote! { #complex_expr };
    println!("{}\n", expr_tokens);

    // Demonstrate interpolation with repetition
    println!("Generated Method Chain:");
    let methods = vec![
        ("filter", vec!["predicate"]),
        ("map", vec!["transform"]),
        ("collect", vec![]),
    ];

    let chain_tokens = generate_method_chain("iter", &methods);
    println!("{}\n", chain_tokens);

    // Show how quote! preserves formatting
    println!("Quote with Custom Formatting:");
    let formatted = quote! {
        impl MyTrait for MyStruct {
            fn method(&self) -> Result<(), Error> {
                // This is a comment that will be preserved
                let result = self.do_something()?;

                if result.is_valid() {
                    Ok(())
                } else {
                    Err(Error::Invalid)
                }
            }
        }
    };
    println!("{}", formatted);
}
