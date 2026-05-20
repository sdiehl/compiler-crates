use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{self, Config};
use codespan_reporting_example::{
    create_info, create_warning, CompilerError, DiagnosticEngine, Project, Type,
};
use termcolor::{ColorChoice, StandardStream};

fn main() {
    println!("=== Basic Diagnostic Reporting ===");
    demonstrate_basic_diagnostics();

    println!("\n=== Type Error Reporting ===");
    demonstrate_type_errors();

    println!("\n=== Parse Error with Hints ===");
    demonstrate_parse_errors();

    println!("\n=== Multi-file Project Errors ===");
    demonstrate_multi_file_errors();

    println!("\n=== Warning and Info Messages ===");
    demonstrate_severity_levels();

    println!("\n=== Complex Diagnostic with Multiple Labels ===");
    demonstrate_complex_diagnostic();
}

fn demonstrate_basic_diagnostics() {
    let mut files = SimpleFiles::new();

    let file_id = files.add(
        "example.ml",
        r#"let add x y = x + y
let result = add 5 "hello""#,
    );

    let diagnostic = Diagnostic::error()
        .with_message("type error in function call")
        .with_labels(vec![
            Label::primary(file_id, 40..47).with_message("expected `int`, found `string`"),
            Label::secondary(file_id, 14..15).with_message("parameter `y` defined here as `int`"),
        ])
        .with_notes(vec!["function `add` expects two integers".to_string()]);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = Config::default();
    term::emit_to_write_style(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
}

fn demonstrate_type_errors() {
    let mut engine = DiagnosticEngine::new();

    let file_id = engine.add_file(
        "types.ml".to_string(),
        r#"let f x = x + 1
let g y = y ^ " world"
let h = f "not a number""#
            .to_string(),
    );

    let error = CompilerError::TypeMismatch {
        expected: Type::Int,
        found: Type::String,
        location: 50..64,
    };

    engine.emit_diagnostic(error.to_diagnostic(file_id));
}

fn demonstrate_parse_errors() {
    let mut engine = DiagnosticEngine::new();

    let source = r#"let x = 5 +
let y = 10"#;

    let file_id = engine.add_file("parse_error.ml".to_string(), source.to_string());

    let error = CompilerError::ParseError {
        message: "expected expression after `+`".to_string(),
        location: 11..11,
        hint: Some("every binary operator needs a right-hand side expression".to_string()),
    };

    engine.emit_diagnostic(error.to_diagnostic(file_id));
}

fn demonstrate_multi_file_errors() {
    let mut project = Project::new();

    // Add multiple files to the project
    project.add_file(
        "src/types.ml".to_string(),
        r#"type person = { name: string; age: int }
let p = { name = 42; age = "old" }"#
            .to_string(),
    );

    project.add_file(
        "src/undefined.ml".to_string(),
        r#"let calculate_area radius =
  let lenght = 2.0 *. pi *. radius in
  length"#
            .to_string(),
    );

    // Compile and display all errors
    let _ = project.compile();
}

fn demonstrate_severity_levels() {
    let mut files = SimpleFiles::new();

    let file_id = files.add(
        "warnings.rs",
        r#"fn unused_function() {
    println!("This is never called");
}

fn main() {
    let x = 5;
    let x = 10; // Shadowing
}"#,
    );

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = Config::default();

    // Warning
    let warning = create_warning(
        file_id,
        "unused function",
        3..18,
        Some("consider removing this function or adding `#[allow(dead_code)]`".to_string()),
    );
    term::emit_to_write_style(&mut writer.lock(), &config, &files, &warning).unwrap();

    // Info
    let info = create_info(file_id, "variable shadowing detected", 96..106);
    term::emit_to_write_style(&mut writer.lock(), &config, &files, &info).unwrap();
}

fn demonstrate_complex_diagnostic() {
    let mut files = SimpleFiles::new();

    let file_id = files.add(
        "complex.rs",
        r#"impl Display for Person {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} ({})", self.name, self.age)
    }
}

impl Display for Person {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}: {} years old", self.name, self.age)
    }
}"#,
    );

    let diagnostic = CompilerError::DuplicateDefinition {
        name: "Display for Person".to_string(),
        first_location: 0..93,
        second_location: 96..206,
    }
    .to_diagnostic(file_id);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = Config::default();
    term::emit_to_write_style(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
}
