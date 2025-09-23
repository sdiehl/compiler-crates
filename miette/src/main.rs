use miette::{IntoDiagnostic, Result};
use miette_example::{
    create_diagnostic, BorrowError, CompilationErrors, CyclicImportError, NonExhaustiveMatch,
    ParseError, Type, TypeMismatchError, UndefinedVariableError,
};

fn main() -> Result<()> {
    // Demonstrating different error scenarios
    println!("=== Miette Diagnostic Examples ===\n");

    // Parse error
    if let Err(e) = demonstrate_parse_error() {
        println!("Parse Error Example:");
        println!("{:?}\n", e);
    }

    // Type mismatch
    if let Err(e) = demonstrate_type_mismatch() {
        println!("Type Mismatch Example:");
        println!("{:?}\n", e);
    }

    // Undefined variable with suggestions
    if let Err(e) = demonstrate_undefined_variable() {
        println!("Undefined Variable Example:");
        println!("{:?}\n", e);
    }

    // Multiple errors
    if let Err(e) = demonstrate_multiple_errors() {
        println!("Multiple Errors Example:");
        println!("{:?}\n", e);
    }

    // Borrow checker error
    if let Err(e) = demonstrate_borrow_error() {
        println!("Borrow Checker Example:");
        println!("{:?}\n", e);
    }

    // Pattern matching exhaustiveness
    if let Err(e) = demonstrate_pattern_matching() {
        println!("Pattern Matching Example:");
        println!("{:?}\n", e);
    }

    // Import cycle
    if let Err(e) = demonstrate_import_cycle() {
        println!("Import Cycle Example:");
        println!("{:?}\n", e);
    }

    // Dynamic diagnostic
    if let Err(e) = demonstrate_dynamic_diagnostic() {
        println!("Dynamic Diagnostic Example:");
        println!("{:?}\n", e);
    }

    // Additional examples
    if let Err(e) = demonstrate_error_conversion() {
        println!("Error Conversion Example:");
        println!("{:?}\n", e);
    }

    if let Err(e) = demonstrate_contextual_help() {
        println!("Contextual Help Example:");
        println!("{:?}\n", e);
    }

    println!("All examples completed!");
    Ok(())
}

fn demonstrate_parse_error() -> Result<()> {
    let source = r#"
fn calculate(x: i32, y: i32) -> i32 {
    let result = x + y
    result
}"#;

    let error = ParseError::new(
        "calculate.rs".to_string(),
        source.to_string(),
        (58, 1).into(), // Position after 'y'
        "semicolon".to_string(),
        "identifier".to_string(),
    )
    .with_context((37, 21).into()); // The let statement

    Err(error)?
}

fn demonstrate_type_mismatch() -> Result<()> {
    let source = r#"
fn process_data() {
    let count: i32 = "42";
    let name: String = 100;
    println!("{} items for {}", count, name);
}"#;

    let error = TypeMismatchError::new(
        "types.rs".to_string(),
        source.to_string(),
        (36, 4).into(), // "42"
        Type::Int,
        Type::String,
    )
    .with_reason((27, 3).into()); // i32 type annotation

    Err(error)?
}

fn demonstrate_undefined_variable() -> Result<()> {
    let source = r#"
fn calculate_area(radius: f64) -> f64 {
    let pi = 3.14159;
    let circumference = 2.0 * pie * radius;
    pi * radius * radius
}"#;

    let similar = vec![
        ("pi", (51, 2).into()),
        ("pie", (0, 0).into()), // Dummy span for the typo itself
    ];

    let error = UndefinedVariableError::new(
        "geometry.rs".to_string(),
        source.to_string(),
        (78, 3).into(), // "pie"
        "pie".to_string(),
        similar,
    );

    Err(error)?
}

fn demonstrate_multiple_errors() -> Result<()> {
    let source = r#"
fn broken_function() {
    let x: i32 = "not a number";
    let y = unknown_var;
    x + y
}"#;

    let mut errors = CompilationErrors::new("broken.rs".to_string(), source.to_string());

    // Add type mismatch error
    let type_error = TypeMismatchError::new(
        "broken.rs".to_string(),
        source.to_string(),
        (38, 14).into(), // "not a number"
        Type::Int,
        Type::String,
    );
    errors.push(type_error);

    // Add undefined variable error
    let undef_error = UndefinedVariableError::new(
        "broken.rs".to_string(),
        source.to_string(),
        (64, 11).into(), // "unknown_var"
        "unknown_var".to_string(),
        vec![],
    );
    errors.push(undef_error);

    Err(errors)?
}

fn demonstrate_borrow_error() -> Result<()> {
    let source = r#"
fn manipulate_data() {
    let mut data = vec![1, 2, 3];
    let first = &mut data;
    let second = &mut data;
    *first += 1;
}"#;

    let error = BorrowError::new(
        "borrow.rs".to_string(),
        source.to_string(),
        (66, 9).into(), // "&mut data" (first)
        (90, 9).into(), // "&mut data" (second)
        "data".to_string(),
    )
    .with_first_use((105, 6).into()); // "*first"

    Err(error)?
}

fn demonstrate_pattern_matching() -> Result<()> {
    let source = r#"
enum Status {
    Ready,
    Running,
    Stopped,
    Error(String),
}

fn handle_status(status: Status) -> &'static str {
    match status {
        Status::Ready => "ready",
        Status::Running => "running",
    }
}"#;

    let missing = vec![
        "Status::Stopped".to_string(),
        "Status::Error(_)".to_string(),
    ];

    let error = NonExhaustiveMatch::new(
        "patterns.rs".to_string(),
        source.to_string(),
        (119, 96).into(), // The entire match expression
        missing,
    );

    Err(error)?
}

fn demonstrate_import_cycle() -> Result<()> {
    let source = r#"
mod graphics {
    use crate::renderer::Renderer;
    use crate::shader::Shader;
}

mod renderer {
    use crate::graphics::Texture;
    use crate::shader::Program;
}

mod shader {
    use crate::renderer::Pipeline;
}"#;

    let modules = vec![
        ("graphics".to_string(), (5, 8).into()),
        ("renderer".to_string(), (96, 8).into()),
        ("shader".to_string(), (182, 6).into()),
    ];

    let error = CyclicImportError::new("modules.rs".to_string(), source.to_string(), modules);

    Err(error)?
}

fn demonstrate_dynamic_diagnostic() -> Result<()> {
    let source = r#"
fn example() {
    unsafe { transmute::<i32, f32>(42) }
}"#;

    let report = create_diagnostic(
        "unsafe.rs".to_string(),
        source.to_string(),
        (22, 34).into(), // The unsafe block
        "Unsafe operation detected".to_string(),
        Some("Consider using safe alternatives like `f32::from_bits`".to_string()),
    );

    Err(report)
}

// Additional example showing conversion from std::error::Error
fn demonstrate_error_conversion() -> Result<()> {
    // This shows how regular errors can be converted to miette diagnostics
    let number = "not_a_number";
    let _parsed: i32 = number.parse().into_diagnostic()?;
    Ok(())
}

// Example with custom help based on context
fn demonstrate_contextual_help() -> Result<()> {
    let source = r#"
struct Config {
    debug: bool,
    threads: usize,
}

fn load_config() -> Config {
    Config {
        debug: "yes",
        threads: "4",
    }
}"#;

    let mut errors = CompilationErrors::new("config.rs".to_string(), source.to_string());

    let bool_error = TypeMismatchError::new(
        "config.rs".to_string(),
        source.to_string(),
        (98, 5).into(), // "yes"
        Type::Bool,
        Type::String,
    );

    let usize_error = TypeMismatchError::new(
        "config.rs".to_string(),
        source.to_string(),
        (124, 3).into(), // "4"
        Type::Struct("usize".to_string()),
        Type::String,
    );

    errors.push(bool_error);
    errors.push(usize_error);

    if !errors.is_empty() {
        Err(errors)?
    }

    Ok(())
}
