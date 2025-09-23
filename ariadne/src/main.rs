use ariadne::{Color, ColorGenerator, Label, Report, ReportKind, Source};
use ariadne_example::{warning_report, CompilerDiagnostic, SourceManager, Type};

fn main() {
    println!("=== Basic Type Error Reporting ===");
    demonstrate_type_error();

    println!("\n=== Unresolved Name with Suggestions ===");
    demonstrate_unresolved_name();

    println!("\n=== Syntax Error with Expected Tokens ===");
    demonstrate_syntax_error();

    println!("\n=== Borrow Checker Error ===");
    demonstrate_borrow_error();

    println!("\n=== Cyclic Dependency Error ===");
    demonstrate_cyclic_dependency();

    println!("\n=== Multi-file Type Error ===");
    demonstrate_multi_file_error();

    println!("\n=== Helper Functions ===");
    demonstrate_helper_functions();

    println!("\n=== Complex Report with Many Labels ===");
    demonstrate_complex_report();
}

fn demonstrate_type_error() {
    let source = r#"fn add(x: int, y: int) -> int {
    x + y
}

fn main() {
    let result = add(5, "hello");
    println!("{}", result);
}"#;

    let diagnostic = CompilerDiagnostic::TypeError {
        expected: Type::Int,
        found: Type::String,
        expr_span: 85..92,
        expected_span: Some(19..22),
        context: "function argument".to_string(),
    };

    diagnostic
        .to_report("main.rs")
        .eprint(("file", Source::from(source)))
        .unwrap();
}

fn demonstrate_unresolved_name() {
    let source = r#"fn calculate_area(radius: float) -> float {
    let circumerence = 2.0 * PI * radius;
    PI * radius * radius
}"#;

    let diagnostic = CompilerDiagnostic::UnresolvedName {
        name: "circumerence".to_string(),
        span: 52..64,
        similar_names: vec!["circumference".to_string()],
        imported_modules: vec!["std::f64::consts::PI".to_string()],
    };

    diagnostic
        .to_report("geometry.rs")
        .eprint(("file", Source::from(source)))
        .unwrap();
}

fn demonstrate_syntax_error() {
    let source = r#"fn process(items: Vec<int>) {
    for item in items {
        if item > 10
            println!("Large: {}", item);
        } else {
            println!("Small: {}", item);
        }
    }
}"#;

    let diagnostic = CompilerDiagnostic::SyntaxError {
        message: "Missing '{' after if condition".to_string(),
        span: 65..65,
        expected: vec!["{".to_string()],
        note: Some("Every if statement requires a block".to_string()),
    };

    diagnostic
        .to_report("syntax.rs")
        .eprint(("file", Source::from(source)))
        .unwrap();
}

fn demonstrate_borrow_error() {
    let source = r#"fn process_data(data: &mut Vec<int>) {
    let first = &data[0];
    data.push(42);  // Error: cannot borrow as mutable
    println!("First element: {}", first);
}"#;

    let diagnostic = CompilerDiagnostic::BorrowError {
        var_name: "data".to_string(),
        first_borrow: 56..64,
        second_borrow: 70..74,
        first_mutable: false,
        second_mutable: true,
    };

    diagnostic
        .to_report("borrow.rs")
        .eprint(("file", Source::from(source)))
        .unwrap();
}

fn demonstrate_cyclic_dependency() {
    // Simulate multiple files
    let module_a = r#"use module_b::helper;

pub fn process() {
    helper();
}"#;

    let _module_b = r#"use module_a::process;

pub fn helper() {
    process();
}"#;

    let diagnostic = CompilerDiagnostic::CyclicDependency {
        modules: vec![
            ("module_a.rs".to_string(), 0..21),
            ("module_b.rs".to_string(), 0..22),
        ],
    };

    // This would normally print across multiple files
    diagnostic
        .to_report("module_a.rs")
        .eprint(("module", Source::from(module_a)))
        .unwrap();
}

fn demonstrate_multi_file_error() {
    let mut manager = SourceManager::new();

    manager.add_file(
        "types.rs".to_string(),
        r#"pub struct User {
    pub name: String,
    pub age: u32,
}"#
        .to_string(),
    );

    manager.add_file(
        "main.rs".to_string(),
        r#"mod types;
use types::User;

fn main() {
    let user = User {
        name: 42,  // Error: expected String
        age: "old",  // Error: expected u32
    };
}"#
        .to_string(),
    );

    // For multi-file errors, create a report directly
    let diagnostic = Report::build(ReportKind::Error, "main.rs", 71)
        .with_message("Multiple type errors in struct initialization")
        .with_label(
            Label::new(("main.rs", 71..73))
                .with_message("Expected String, found integer")
                .with_color(Color::Red),
        )
        .with_label(
            Label::new(("main.rs", 115..120))
                .with_message("Expected u32, found string")
                .with_color(Color::Red),
        )
        .with_note("Struct fields must match their declared types")
        .with_help("Use User { name: \"John\".to_string(), age: 42 }")
        .finish();

    if let Some(source) = manager.get_source("main.rs") {
        diagnostic.eprint(("file", source)).unwrap();
    }
}

fn demonstrate_helper_functions() {
    let source = r#"fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        panic!("Division by zero");
    }
    a / b
}"#;

    // Using the simplified helper functions
    let warning = warning_report(
        "math.rs",
        105..110,
        "Possible division by zero",
        "Division happens here",
    );

    warning.eprint(("static", Source::from(source))).unwrap();
}

fn demonstrate_complex_report() {
    let source = r#"trait Display {
    fn fmt(&self) -> String;
}

struct Point { x: i32, y: i32 }

impl Display for Point {
    fn fmt(&self) -> String {
        format!("({}, {})", self.x, self.y)
    }
}

impl Display for Point {
    fn fmt(&self) -> String {
        format!("Point[x={}, y={}]", self.x, self.y)
    }
}"#;

    let mut colors = ColorGenerator::new();

    Report::build(ReportKind::Error, "traits.rs", 134)
        .with_message("Duplicate trait implementation")
        .with_label(
            Label::new(("traits.rs", 75..135))
                .with_message("First implementation here")
                .with_color(colors.next()),
        )
        .with_label(
            Label::new(("traits.rs", 138..232))
                .with_message("Duplicate implementation here")
                .with_color(colors.next()),
        )
        .with_label(
            Label::new(("traits.rs", 0..40))
                .with_message("Trait 'Display' defined here")
                .with_color(colors.next()),
        )
        .with_note("A type can only have one implementation of a trait")
        .with_help("Consider using different trait names or removing one implementation")
        .finish()
        .eprint(("file", Source::from(source)))
        .unwrap();
}
