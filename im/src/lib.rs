//! Example: persistent typing environments with `im`. Extending a scope is a
//! cheap copy that shares structure with its parent, so there is no undo log
//! and every version of the environment stays valid.

use im::ordmap::DiffItem;
use im::OrdMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty {
    Int,
    Bool,
    Fun(Box<Ty>, Box<Ty>),
}

#[derive(Clone, Debug)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Var(String),
    Lam(String, Ty, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

/// The typing context. `OrdMap` iterates in key order, which keeps
/// diagnostics that list the environment deterministic.
pub type Env = OrdMap<String, Ty>;

/// Simply typed lambda calculus. Binders extend `env` with `update`, which
/// returns a new map and leaves the caller's untouched, so leaving a scope is
/// just returning.
pub fn check(env: &Env, e: &Expr) -> Result<Ty, String> {
    match e {
        Expr::Int(_) => Ok(Ty::Int),
        Expr::Bool(_) => Ok(Ty::Bool),
        Expr::Var(x) => env.get(x).cloned().ok_or(format!("unbound {}", x)),
        Expr::Lam(x, t, body) => {
            let r = check(&env.update(x.clone(), t.clone()), body)?;
            Ok(Ty::Fun(Box::new(t.clone()), Box::new(r)))
        }
        Expr::App(f, a) => match check(env, f)? {
            Ty::Fun(p, r) if *p == check(env, a)? => Ok(*r),
            t => Err(format!("cannot apply {:?}", t)),
        },
        Expr::Let(x, v, body) => {
            let t = check(env, v)?;
            check(&env.update(x.clone(), t), body)
        }
        Expr::If(c, t, f) => {
            if check(env, c)? != Ty::Bool {
                return Err("condition must be Bool".into());
            }
            let (t, f) = (check(env, t)?, check(env, f)?);
            if t == f {
                Ok(t)
            } else {
                Err(format!("branches differ: {:?} vs {:?}", t, f))
            }
        }
    }
}

/// A REPL session that keeps every environment it has ever had. Each entry
/// shares all but the changed path with its predecessor, so the whole history
/// costs little more than the latest version.
#[derive(Default)]
pub struct Session {
    history: Vec<Env>,
}

impl Session {
    pub fn env(&self) -> Env {
        self.history.last().cloned().unwrap_or_default()
    }

    pub fn define(&mut self, name: &str, e: &Expr) -> Result<Ty, String> {
        let env = self.env();
        let t = check(&env, e)?;
        self.history.push(env.update(name.into(), t.clone()));
        Ok(t)
    }

    pub fn undo(&mut self) {
        self.history.pop();
    }

    /// What the last definition changed, found by walking only the parts of
    /// the two trees that are not shared.
    pub fn last_change(&self) -> Vec<String> {
        let n = self.history.len();
        let prev = if n > 1 {
            self.history[n - 2].clone()
        } else {
            Env::new()
        };
        prev.diff(&self.env())
            .map(|d| match d {
                DiffItem::Add(k, t) => format!("+ {} : {:?}", k, t),
                DiffItem::Update { new: (k, t), .. } => format!("~ {} : {:?}", k, t),
                DiffItem::Remove(k, _) => format!("- {}", k),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn var(x: &str) -> Box<Expr> {
        Box::new(Expr::Var(x.into()))
    }

    #[test]
    fn inner_scope_does_not_leak() {
        let env = Env::new().update("x".into(), Ty::Int);
        let e = Expr::App(
            Box::new(Expr::Lam("x".into(), Ty::Bool, var("x"))),
            Box::new(Expr::Bool(true)),
        );
        assert_eq!(check(&env, &e), Ok(Ty::Bool));
        assert_eq!(env.get("x"), Some(&Ty::Int));
    }

    #[test]
    fn session_history_and_undo() {
        let mut s = Session::default();
        s.define("x", &Expr::Int(1)).unwrap();
        s.define("x", &Expr::Bool(true)).unwrap();
        assert_eq!(s.last_change(), vec!["~ x : Bool"]);
        s.undo();
        assert_eq!(s.env().get("x"), Some(&Ty::Int));
        assert!(s
            .define("y", &Expr::If(var("x"), var("x"), var("x")))
            .is_err());
    }
}
