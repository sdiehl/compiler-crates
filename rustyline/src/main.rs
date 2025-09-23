use rustyline_example::{create_editor, process_command};

fn main() -> rustyline::Result<()> {
    println!("=== Compiler REPL ===");
    println!("Type 'help' for available commands or 'quit' to exit.");
    println!();

    let mut editor = create_editor()?;

    loop {
        let readline = editor.readline("compiler> ");
        match readline {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                editor.add_history_entry(&line)?;

                if !process_command(&line, editor.helper().unwrap()) {
                    break;
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("CTRL-C pressed. Use 'quit' to exit.");
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("CTRL-D pressed. Exiting.");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    editor.save_history("compiler_history.txt")?;
    println!("History saved. Goodbye!");

    Ok(())
}
