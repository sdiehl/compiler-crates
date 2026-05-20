//! Driver showing the pretty `Debug` output that `thiserror-context` produces
//! once context layers and cross-phase conversions are wired in.

use thiserror_example::{compile, lex_identifier, parse_let_binding};

fn main() {
    println!("=== Compiler Error Handling with thiserror-context ===\n");

    println!("1. Lexer error with a single context layer:");
    if let Err(e) = lex_identifier("1bad", 0) {
        println!("Display: {e}");
        println!("Debug:\n{e:?}\n");
    }

    println!("2. Parser error reporting a missing token:");
    if let Err(e) = parse_let_binding(&["if"]) {
        println!("Display: {e}");
        println!("Debug:\n{e:?}\n");
    }

    println!("3. Lexer error surfacing through the parser:");
    if let Err(e) = parse_let_binding(&["let", "1bad", "=", "x"]) {
        println!("Display: {e}");
        println!("Debug:\n{e:?}\n");
    }

    println!("4. Same error one level up, wrapped as a CompilerError:");
    if let Err(e) = compile("let 1bad = x") {
        println!("Display: {e}");
        println!("Debug:\n{e:?}\n");
    }
}
