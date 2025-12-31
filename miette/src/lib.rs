#![allow(unused_assignments)]

use std::fmt;

use miette::{Diagnostic, LabeledSpan, NamedSource, SourceSpan};
use thiserror::Error;

/// Type representation for our compiler
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Array(Box<Type>),
    Function(Vec<Type>, Box<Type>),
    Struct(String),
    Never,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Array(elem) => write!(f, "[{}]", elem),
            Type::Function(params, ret) => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Struct(name) => write!(f, "{}", name),
            Type::Never => write!(f, "!"),
        }
    }
}

/// Parser error with detailed diagnostics
#[derive(Error, Debug, Diagnostic)]
#[error("Parse error in {filename}")]
#[diagnostic(
    code(compiler::parse::syntax_error),
    url(docsrs),
    help("Check for missing semicolons, unmatched brackets, or typos in keywords")
)]
pub struct ParseError {
    #[source_code]
    src: NamedSource<String>,

    #[label("Expected {expected} but found {found}")]
    err_span: SourceSpan,

    expected: String,
    found: String,
    filename: String,

    #[label("Parsing started here")]
    context_span: Option<SourceSpan>,
}

impl ParseError {
    pub fn new(
        filename: String,
        source: String,
        span: SourceSpan,
        expected: String,
        found: String,
    ) -> Self {
        Self {
            src: NamedSource::new(filename.clone(), source),
            err_span: span,
            expected,
            found,
            filename,
            context_span: None,
        }
    }

    pub fn with_context(mut self, span: SourceSpan) -> Self {
        self.context_span = Some(span);
        self
    }
}

/// Type mismatch error with rich diagnostics
#[derive(Error, Debug, Diagnostic)]
#[error("Type mismatch in expression")]
#[diagnostic(
    code(compiler::typecheck::type_mismatch),
    url("https://example.com/errors/type-mismatch"),
    severity(Error)
)]
pub struct TypeMismatchError {
    #[source_code]
    src: NamedSource<String>,

    #[label(primary, "Expected type `{expected}` but found `{actual}`")]
    expr_span: SourceSpan,

    #[label("Expected due to this")]
    reason_span: Option<SourceSpan>,

    expected: Type,
    actual: Type,

    #[help]
    suggestion: Option<String>,
}

impl TypeMismatchError {
    pub fn new(
        filename: String,
        source: String,
        expr_span: SourceSpan,
        expected: Type,
        actual: Type,
    ) -> Self {
        let suggestion = match (&expected, &actual) {
            (Type::String, Type::Int) => Some("Try using `.to_string()` to convert".to_string()),
            (Type::Int, Type::String) => {
                Some("Try using `.parse::<i32>()?` to convert".to_string())
            }
            (Type::Float, Type::Int) => Some("Try using `as f64` to convert".to_string()),
            _ => None,
        };

        Self {
            src: NamedSource::new(filename, source),
            expr_span,
            reason_span: None,
            expected,
            actual,
            suggestion,
        }
    }

    pub fn with_reason(mut self, span: SourceSpan) -> Self {
        self.reason_span = Some(span);
        self
    }
}

/// Undefined variable error with suggestions
#[derive(Error, Debug, Diagnostic)]
#[error("Undefined variable `{name}`")]
#[diagnostic(
    code(compiler::resolve::undefined_variable),
    help("Did you mean {suggestions}?")
)]
pub struct UndefinedVariableError {
    #[source_code]
    src: NamedSource<String>,

    #[label(primary, "Not found in this scope")]
    var_span: SourceSpan,

    name: String,
    suggestions: String,

    #[related]
    similar_vars: Vec<SimilarVariable>,
}

/// Similar variable found in scope
#[derive(Error, Debug, Diagnostic)]
#[error("Similar variable `{name}` defined here")]
#[diagnostic(severity(Warning))]
struct SimilarVariable {
    #[label]
    span: SourceSpan,
    name: String,
}

impl UndefinedVariableError {
    pub fn new(
        filename: String,
        source: String,
        span: SourceSpan,
        name: String,
        similar: Vec<(&str, SourceSpan)>,
    ) -> Self {
        let suggestions = similar
            .iter()
            .map(|(name, _)| format!("`{}`", name))
            .collect::<Vec<_>>()
            .join(", ");

        let similar_vars = similar
            .into_iter()
            .map(|(name, span)| SimilarVariable {
                span,
                name: name.to_string(),
            })
            .collect();

        Self {
            src: NamedSource::new(filename, source),
            var_span: span,
            name,
            suggestions,
            similar_vars,
        }
    }
}

/// Multiple errors collected together
#[derive(Error, Debug, Diagnostic)]
#[error("Multiple errors occurred during compilation")]
#[diagnostic(
    code(compiler::multiple_errors),
    help("Fix the errors in order, as later errors may be caused by earlier ones")
)]
pub struct CompilationErrors {
    #[source_code]
    src: NamedSource<String>,

    #[related]
    errors: Vec<Box<dyn Diagnostic + Send + Sync>>,

    error_count: usize,
    warning_count: usize,
}

impl CompilationErrors {
    pub fn new(filename: String, source: String) -> Self {
        Self {
            src: NamedSource::new(filename, source),
            errors: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn push<E: Diagnostic + Send + Sync + 'static>(&mut self, error: E) {
        match error.severity() {
            Some(miette::Severity::Warning) => self.warning_count += 1,
            _ => self.error_count += 1,
        }
        self.errors.push(Box::new(error));
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Borrow checker error
#[derive(Error, Debug, Diagnostic)]
#[error("Cannot borrow `{variable}` as mutable more than once")]
#[diagnostic(
    code(compiler::borrow_check::multiple_mutable),
    url(docsrs),
    help("Consider using RefCell for interior mutability")
)]
pub struct BorrowError {
    #[source_code]
    src: NamedSource<String>,

    #[label(primary, "Second mutable borrow occurs here")]
    second_borrow: SourceSpan,

    #[label("First mutable borrow occurs here")]
    first_borrow: SourceSpan,

    #[label("First borrow later used here")]
    first_use: Option<SourceSpan>,

    variable: String,
}

impl BorrowError {
    pub fn new(
        filename: String,
        source: String,
        first_borrow: SourceSpan,
        second_borrow: SourceSpan,
        variable: String,
    ) -> Self {
        Self {
            src: NamedSource::new(filename, source),
            second_borrow,
            first_borrow,
            first_use: None,
            variable,
        }
    }

    pub fn with_first_use(mut self, span: SourceSpan) -> Self {
        self.first_use = Some(span);
        self
    }
}

/// Pattern matching exhaustiveness error
#[derive(Error, Debug, Diagnostic)]
#[error("Non-exhaustive patterns")]
#[diagnostic(code(compiler::pattern_match::non_exhaustive))]
pub struct NonExhaustiveMatch {
    #[source_code]
    src: NamedSource<String>,

    #[label(primary, "Pattern match is non-exhaustive")]
    match_span: SourceSpan,

    #[label(collection, "Missing pattern")]
    missing_patterns: Vec<LabeledSpan>,

    #[help]
    missing_list: String,
}

impl NonExhaustiveMatch {
    pub fn new(
        filename: String,
        source: String,
        match_span: SourceSpan,
        missing: Vec<String>,
    ) -> Self {
        let missing_patterns = missing
            .iter()
            .map(|_pattern| LabeledSpan::underline(match_span))
            .collect();

        let missing_list =
            format!(
            "Missing patterns: {}\n\nEnsure all cases are covered or add a wildcard pattern `_`",
            missing.iter().map(|p| format!("`{}`", p)).collect::<Vec<_>>().join(", ")
        );

        Self {
            src: NamedSource::new(filename, source),
            match_span,
            missing_patterns,
            missing_list,
        }
    }
}

/// Import cycle detection
#[derive(Error, Debug, Diagnostic)]
#[error("Circular dependency detected")]
#[diagnostic(code(compiler::imports::cycle), severity(Error))]
pub struct CyclicImportError {
    #[source_code]
    src: NamedSource<String>,

    #[label(collection, "Module in cycle")]
    cycle_spans: Vec<LabeledSpan>,

    #[help]
    help_text: String,
}

impl CyclicImportError {
    pub fn new(filename: String, source: String, modules: Vec<(String, SourceSpan)>) -> Self {
        let cycle_spans = modules
            .iter()
            .enumerate()
            .map(|(i, (name, span))| {
                let next = &modules[(i + 1) % modules.len()].0;
                LabeledSpan::new(
                    Some(format!("`{}` imports `{}`", name, next)),
                    span.offset(),
                    span.len(),
                )
            })
            .collect();

        let module_list = modules
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(" -> ");

        Self {
            src: NamedSource::new(filename, source),
            cycle_spans,
            help_text: format!("Break the cycle: {} -> ...", module_list),
        }
    }
}

/// Deprecated feature warning
#[derive(Error, Debug, Diagnostic)]
#[error("Use of deprecated feature `{feature}`")]
#[diagnostic(code(compiler::deprecated), severity(Warning))]
pub struct DeprecationWarning {
    #[source_code]
    src: NamedSource<String>,

    #[label(primary, "Deprecated since version {since}")]
    usage_span: SourceSpan,

    feature: String,
    since: String,

    #[help]
    alternative: String,
}

impl DeprecationWarning {
    pub fn new(
        filename: String,
        source: String,
        usage_span: SourceSpan,
        feature: String,
        since: String,
        alternative: String,
    ) -> Self {
        Self {
            src: NamedSource::new(filename, source),
            usage_span,
            feature,
            since,
            alternative,
        }
    }
}

/// Dynamic diagnostic creation
pub fn create_diagnostic(
    filename: String,
    source: String,
    span: SourceSpan,
    message: String,
    help: Option<String>,
) -> miette::Report {
    let labels = vec![LabeledSpan::underline(span)];

    miette::miette!(
        labels = labels,
        help = help.unwrap_or_default(),
        "{}",
        message
    )
    .with_source_code(NamedSource::new(filename, source))
}

/// Syntax highlighting support
pub fn create_highlighted_error(
    filename: String,
    source: String,
    span: SourceSpan,
) -> impl Diagnostic {
    let src = NamedSource::new(filename, source).with_language("rust");

    #[derive(Error, Debug, Diagnostic)]
    #[error("Syntax error in Rust code")]
    #[diagnostic(code(compiler::syntax))]
    struct HighlightedError {
        #[source_code]
        src: NamedSource<String>,

        #[label("Invalid syntax here")]
        span: SourceSpan,
    }

    HighlightedError { src, span }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        let func_type = Type::Function(vec![Type::Int, Type::String], Box::new(Type::Bool));
        assert_eq!(func_type.to_string(), "fn(int, string) -> bool");
    }

    #[test]
    fn test_parse_error_creation() {
        let error = ParseError::new(
            "test.rs".to_string(),
            "let x = ;".to_string(),
            (8, 1).into(),
            "expression".to_string(),
            "semicolon".to_string(),
        );

        assert!(error.to_string().contains("Parse error"));
    }

    #[test]
    fn test_multiple_errors() {
        let mut errors = CompilationErrors::new("test.rs".to_string(), "code".to_string());
        assert!(errors.is_empty());

        let parse_err = ParseError::new(
            "test.rs".to_string(),
            "code".to_string(),
            (0, 4).into(),
            "identifier".to_string(),
            "keyword".to_string(),
        );
        errors.push(parse_err);

        assert!(!errors.is_empty());
        assert_eq!(errors.error_count, 1);
        assert_eq!(errors.warning_count, 0);
    }
}
