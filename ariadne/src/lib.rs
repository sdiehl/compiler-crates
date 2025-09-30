use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

use ariadne::{Color, ColorGenerator, Fmt, Label, Report, ReportKind, Source};

/// A source file with name and content
pub struct SourceFile {
    pub name: String,
    pub content: String,
}

/// Source code manager for multi-file projects
pub struct SourceManager {
    files: HashMap<String, SourceFile>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, name: String, content: String) {
        self.files.insert(
            name.clone(),
            SourceFile {
                name: name.clone(),
                content,
            },
        );
    }

    pub fn get_source(&self, file: &str) -> Option<Source> {
        self.files
            .get(file)
            .map(|f| Source::from(f.content.clone()))
    }
}

impl Default for SourceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Type representation for our language
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Array(Box<Type>),
    Function(Vec<Type>, Box<Type>),
    Struct(String, Vec<(String, Type)>),
    Generic(String),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Array(elem) => write!(f, "{}[]", elem),
            Type::Function(params, ret) => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Struct(name, _) => write!(f, "{}", name),
            Type::Generic(name) => write!(f, "'{}", name),
        }
    }
}

/// Compiler diagnostics with rich information
#[derive(Debug, Clone)]
pub enum CompilerDiagnostic {
    TypeError {
        expected: Type,
        found: Type,
        expr_span: Range<usize>,
        expected_span: Option<Range<usize>>,
        context: String,
    },
    UnresolvedName {
        name: String,
        span: Range<usize>,
        similar_names: Vec<String>,
        imported_modules: Vec<String>,
    },
    SyntaxError {
        message: String,
        span: Range<usize>,
        expected: Vec<String>,
        note: Option<String>,
    },
    BorrowError {
        var_name: String,
        first_borrow: Range<usize>,
        second_borrow: Range<usize>,
        first_mutable: bool,
        second_mutable: bool,
    },
    CyclicDependency {
        modules: Vec<(String, Range<usize>)>,
    },
}

impl CompilerDiagnostic {
    pub fn to_report(&self, _file_id: &str) -> Report<'static, (&'static str, Range<usize>)> {
        match self {
            CompilerDiagnostic::TypeError {
                expected,
                found,
                expr_span,
                expected_span,
                context,
            } => {
                let mut report = Report::build(ReportKind::Error, ("file", expr_span.clone()))
                    .with_message(format!("Type mismatch in {}", context))
                    .with_label(
                        Label::new(("file", expr_span.clone()))
                            .with_message(format!(
                                "Expected {}, found {}",
                                expected.to_string().fg(Color::Green),
                                found.to_string().fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    );

                if let Some(expected_span) = expected_span {
                    report = report.with_label(
                        Label::new(("file", expected_span.clone()))
                            .with_message("Expected because of this")
                            .with_color(Color::Blue),
                    );
                }

                report
                    .with_note(format!(
                        "Cannot convert {} to {}",
                        found.to_string().fg(Color::Red),
                        expected.to_string().fg(Color::Green)
                    ))
                    .finish()
            }

            CompilerDiagnostic::UnresolvedName {
                name,
                span,
                similar_names,
                imported_modules,
            } => {
                let mut report = Report::build(ReportKind::Error, ("file", span.clone()))
                    .with_message(format!("Cannot find '{}' in scope", name))
                    .with_label(
                        Label::new(("file", span.clone()))
                            .with_message("Not found")
                            .with_color(Color::Red),
                    );

                if !similar_names.is_empty() {
                    let suggestions = similar_names
                        .iter()
                        .map(|s| s.fg(Color::Green).to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    report = report.with_help(format!("Did you mean: {}?", suggestions));
                }

                if !imported_modules.is_empty() {
                    report = report.with_note(format!(
                        "Available in modules: {}",
                        imported_modules.join(", ")
                    ));
                }

                report.finish()
            }

            CompilerDiagnostic::SyntaxError {
                message,
                span,
                expected,
                note,
            } => {
                let mut report = Report::build(ReportKind::Error, ("file", span.clone()))
                    .with_message("Syntax error")
                    .with_label(
                        Label::new(("file", span.clone()))
                            .with_message(message)
                            .with_color(Color::Red),
                    );

                if !expected.is_empty() {
                    report = report.with_help(format!(
                        "Expected one of: {}",
                        expected
                            .iter()
                            .map(|e| format!("'{}'", e).fg(Color::Green).to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }

                if let Some(note) = note {
                    report = report.with_note(note);
                }

                report.finish()
            }

            CompilerDiagnostic::BorrowError {
                var_name,
                first_borrow,
                second_borrow,
                first_mutable,
                second_mutable,
            } => {
                let (first_kind, first_color) = if *first_mutable {
                    ("mutable", Color::Yellow)
                } else {
                    ("immutable", Color::Blue)
                };

                let (second_kind, second_color) = if *second_mutable {
                    ("mutable", Color::Yellow)
                } else {
                    ("immutable", Color::Blue)
                };

                Report::build(ReportKind::Error, ("file", second_borrow.clone()))
                    .with_message(format!("Cannot borrow '{}' as {}", var_name, second_kind))
                    .with_label(
                        Label::new(("file", first_borrow.clone()))
                            .with_message(format!("First {} borrow occurs here", first_kind))
                            .with_color(first_color),
                    )
                    .with_label(
                        Label::new(("file", second_borrow.clone()))
                            .with_message(format!(
                                "Second {} borrow occurs here",
                                second_kind
                            ))
                            .with_color(second_color),
                    )
                    .with_note("Cannot have multiple mutable borrows or a mutable borrow with immutable borrows")
                    .finish()
            }

            CompilerDiagnostic::CyclicDependency { modules } => {
                let mut colors = ColorGenerator::new();
                let mut report = Report::build(ReportKind::Error, ("module", modules[0].1.clone()))
                    .with_message("Cyclic module dependency detected");

                for (i, (module, span)) in modules.iter().enumerate() {
                    let color = colors.next();
                    let next_module = &modules[(i + 1) % modules.len()].0;
                    report = report.with_label(
                        Label::new(("module", span.clone()))
                            .with_message(format!("'{}' imports '{}'", module, next_module))
                            .with_color(color),
                    );
                }

                report
                    .with_note("Remove one of the imports to break the cycle")
                    .finish()
            }
        }
    }
}

/// Language server protocol-style diagnostics
pub struct LspDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
    pub message: String,
    pub related_information: Vec<RelatedInformation>,
    pub tags: Vec<DiagnosticTag>,
}

#[derive(Debug, Clone, Copy)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone)]
pub struct RelatedInformation {
    pub location: (String, Range<usize>),
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum DiagnosticTag {
    Unnecessary,
    Deprecated,
}

/// Convert compiler diagnostics to LSP format
pub fn to_lsp_diagnostic(diagnostic: &CompilerDiagnostic, _file: &str) -> LspDiagnostic {
    match diagnostic {
        CompilerDiagnostic::TypeError { .. } => LspDiagnostic {
            severity: DiagnosticSeverity::Error,
            code: Some("E0308".to_string()),
            message: "Type mismatch".to_string(),
            related_information: vec![],
            tags: vec![],
        },
        CompilerDiagnostic::UnresolvedName { name, .. } => LspDiagnostic {
            severity: DiagnosticSeverity::Error,
            code: Some("E0425".to_string()),
            message: format!("Cannot find '{}' in scope", name),
            related_information: vec![],
            tags: vec![],
        },
        CompilerDiagnostic::SyntaxError { message, .. } => LspDiagnostic {
            severity: DiagnosticSeverity::Error,
            code: None,
            message: message.clone(),
            related_information: vec![],
            tags: vec![],
        },
        CompilerDiagnostic::BorrowError { var_name, .. } => LspDiagnostic {
            severity: DiagnosticSeverity::Error,
            code: Some("E0502".to_string()),
            message: format!("Cannot borrow '{}'", var_name),
            related_information: vec![],
            tags: vec![],
        },
        CompilerDiagnostic::CyclicDependency { modules } => LspDiagnostic {
            severity: DiagnosticSeverity::Error,
            code: Some("E0391".to_string()),
            message: "Cyclic dependency detected".to_string(),
            related_information: modules
                .iter()
                .map(|(module, span)| RelatedInformation {
                    location: (module.clone(), span.clone()),
                    message: format!("Module '{}' is part of the cycle", module),
                })
                .collect(),
            tags: vec![],
        },
    }
}

/// Helper function to create error reports
pub fn error_report(
    _file: &str,
    span: Range<usize>,
    message: &str,
    label_msg: &str,
) -> Report<'static, (&'static str, Range<usize>)> {
    Report::build(ReportKind::Error, ("static", span.clone()))
        .with_message(message)
        .with_label(
            Label::new(("static", span))
                .with_message(label_msg)
                .with_color(Color::Red),
        )
        .finish()
}

/// Helper function to create warning reports
pub fn warning_report(
    _file: &str,
    span: Range<usize>,
    message: &str,
    label_msg: &str,
) -> Report<'static, (&'static str, Range<usize>)> {
    Report::build(ReportKind::Warning, ("static", span.clone()))
        .with_message(message)
        .with_label(
            Label::new(("static", span))
                .with_message(label_msg)
                .with_color(Color::Yellow),
        )
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        let int_array = Type::Array(Box::new(Type::Int));
        assert_eq!(int_array.to_string(), "int[]");

        let func = Type::Function(vec![Type::Int, Type::String], Box::new(Type::Bool));
        assert_eq!(func.to_string(), "(int, string) -> bool");
    }

    #[test]
    fn test_source_manager() {
        let mut manager = SourceManager::new();
        manager.add_file("test.rs".to_string(), "let x = 5;".to_string());
        assert!(manager.get_source("test.rs").is_some());
        assert!(manager.get_source("missing.rs").is_none());
    }

    #[test]
    fn test_error_report() {
        let report = error_report("test.rs", 10..15, "Type mismatch", "Expected int");

        // Just ensure it builds without panic
        let _ = format!("{:?}", report);
    }
}
