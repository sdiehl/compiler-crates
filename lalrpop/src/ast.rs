/// AST for a simple expression language
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
    Negate(Box<Expr>),
    Variable(String),
    Call(String, Vec<Expr>),
}

impl Expr {
    /// Evaluate the expression
    pub fn eval(&self) -> f64 {
        match self {
            Expr::Number(n) => *n,
            Expr::Add(l, r) => l.eval() + r.eval(),
            Expr::Subtract(l, r) => l.eval() - r.eval(),
            Expr::Multiply(l, r) => l.eval() * r.eval(),
            Expr::Divide(l, r) => l.eval() / r.eval(),
            Expr::Negate(e) => -e.eval(),
            Expr::Variable(_) => panic!("Cannot evaluate variable without context"),
            Expr::Call(name, args) => match name.as_str() {
                "max" if args.len() == 2 => f64::max(args[0].eval(), args[1].eval()),
                "min" if args.len() == 2 => f64::min(args[0].eval(), args[1].eval()),
                "sqrt" if args.len() == 1 => args[0].eval().sqrt(),
                _ => panic!("Unknown function: {}", name),
            },
        }
    }
}

/// Statement types for a simple language
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expr),
    Assignment(String, Expr),
    Print(Expr),
    If(Expr, Vec<Statement>, Option<Vec<Statement>>),
    While(Expr, Vec<Statement>),
}

/// A complete program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}
