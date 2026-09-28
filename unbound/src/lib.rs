//! Example: untyped lambda calculus in locally nameless form with `unbound`.
//! Alpha equivalence, capture-avoiding substitution, normalization and a
//! printer that never captures, with no hand-written renaming.

use unbound::prelude::*;

/// Terms of the untyped lambda calculus. The derives give alpha equivalence
/// and substitution, with `Var` picked out as the variable case by name.
#[derive(Clone, Debug, Alpha, Subst)]
pub enum Expr {
    Var(Name<Expr>),
    Lam(Bind<Name<Expr>, Box<Expr>>),
    App(Box<Expr>, Box<Expr>),
}

/// A variable by spelling. `Name::global` returns the same name for the same
/// string, so building terms bottom-up resolves scope like a parser would.
pub fn var(x: &str) -> Expr {
    Expr::Var(Name::global(x))
}

/// Abstract `x` in `body`, closing every free occurrence into a de Bruijn
/// index.
pub fn lam(x: &str, body: Expr) -> Expr {
    Expr::Lam(bind(Name::global(x), Box::new(body)))
}

pub fn app(f: Expr, a: Expr) -> Expr {
    Expr::App(Box::new(f), Box::new(a))
}

/// Normal-order reduction to beta normal form. `instantiate` substitutes the
/// argument straight into the closed body, so there is nothing to freshen.
pub fn normalize(e: &Expr) -> Expr {
    match e {
        Expr::Var(_) => e.clone(),
        Expr::Lam(b) => {
            let (x, body) = b.unbind_ref();
            Expr::Lam(bind(x, Box::new(normalize(&body))))
        }
        Expr::App(f, a) => match normalize(f) {
            Expr::Lam(b) => normalize(&b.instantiate(a.as_ref())),
            f => app(f, normalize(a)),
        },
    }
}

/// Print a term, renaming a binder only when its spelling would capture a
/// free name the body actually uses.
pub fn pretty(e: &Expr) -> String {
    go(e, &mut NameScope::new(&e.fv()))
}

fn go(e: &Expr, scope: &mut NameScope) -> String {
    match e {
        Expr::Var(x) => scope.get(x).to_owned(),
        Expr::Lam(b) => {
            let (x, body) = b.unbind_ref();
            let s = scope.bind(&x, &body.fv());
            let out = format!("\\{}. {}", s, go(&body, scope));
            scope.pop();
            out
        }
        Expr::App(f, a) => format!("({} {})", go(f, scope), go(a, scope)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_equivalence_ignores_binder_names() {
        assert!(lam("x", var("x")).aeq(&lam("y", var("y"))));
        assert!(!lam("x", var("y")).aeq(&lam("y", var("y"))));
    }

    #[test]
    fn substitution_does_not_capture() {
        let k = lam("x", lam("y", var("x")));
        let e = normalize(&app(k, var("y")));
        assert!(e.aeq(&lam("z", var("y"))));
        assert_eq!(pretty(&e), "\\y1. y");
    }

    #[test]
    fn skk_is_identity() {
        let s = lam(
            "f",
            lam(
                "g",
                lam("x", app(app(var("f"), var("x")), app(var("g"), var("x")))),
            ),
        );
        let k = lam("x", lam("y", var("x")));
        let i = normalize(&app(app(s, k.clone()), k));
        assert!(i.aeq(&lam("a", var("a"))));
    }
}
