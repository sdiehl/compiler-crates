use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::{HashMap, HashSet};

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::{CmdKind, Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::history::DefaultHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{CompletionType, Config, Context, EditMode, Editor, Helper, Result};

#[derive(Debug, Clone)]
pub struct CompilerCommand {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static str,
}

impl CompilerCommand {
    pub const COMMANDS: &'static [CompilerCommand] = &[
        CompilerCommand {
            name: "load",
            description: "Load a source file",
            args: "<filename>",
        },
        CompilerCommand {
            name: "compile",
            description: "Compile the current module",
            args: "[--optimize] [--debug]",
        },
        CompilerCommand {
            name: "run",
            description: "Run the compiled program",
            args: "[args...]",
        },
        CompilerCommand {
            name: "ast",
            description: "Show the AST",
            args: "[function_name]",
        },
        CompilerCommand {
            name: "ir",
            description: "Show intermediate representation",
            args: "[function_name]",
        },
        CompilerCommand {
            name: "symbols",
            description: "List all symbols",
            args: "[pattern]",
        },
        CompilerCommand {
            name: "type",
            description: "Show type of expression",
            args: "<expression>",
        },
        CompilerCommand {
            name: "help",
            description: "Show help",
            args: "[command]",
        },
        CompilerCommand {
            name: "quit",
            description: "Exit the REPL",
            args: "",
        },
    ];
}

pub struct CompilerREPL {
    pub commands: HashMap<String, CompilerCommand>,
    pub keywords: HashSet<&'static str>,
    pub history_file: String,
    pub completer: CommandCompleter,
    pub highlighter: MatchingBracketHighlighter,
    pub hinter: HistoryHinter,
    pub validator: CompilerValidator,
}

impl Helper for CompilerREPL {}

#[derive(Clone)]
pub struct CommandCompleter {
    commands: Vec<String>,
    keywords: Vec<&'static str>,
}

impl CommandCompleter {
    pub fn new() -> Self {
        let commands = CompilerCommand::COMMANDS
            .iter()
            .map(|cmd| cmd.name.to_string())
            .collect();

        let keywords = vec![
            "fn", "let", "const", "if", "else", "while", "for", "return", "struct", "enum", "impl",
            "trait", "pub", "mod", "use",
        ];

        Self { commands, keywords }
    }
}

impl Default for CommandCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        let line_before_cursor = &line[..pos];
        let words: Vec<&str> = line_before_cursor.split_whitespace().collect();

        if words.is_empty() || (words.len() == 1 && !line_before_cursor.ends_with(' ')) {
            let prefix = words.first().unwrap_or(&"");
            let matches: Vec<Pair> = self
                .commands
                .iter()
                .filter(|cmd| cmd.starts_with(prefix))
                .map(|cmd| Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                })
                .collect();

            Ok((0, matches))
        } else {
            let last_word = words.last().unwrap_or(&"");
            let word_start = line_before_cursor.rfind(last_word).unwrap_or(pos);

            let matches: Vec<Pair> = self
                .keywords
                .iter()
                .filter(|kw| kw.starts_with(last_word))
                .map(|kw| Pair {
                    display: kw.to_string(),
                    replacement: kw.to_string(),
                })
                .collect();

            Ok((word_start, matches))
        }
    }
}

impl Completer for CompilerREPL {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for CompilerREPL {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for CompilerREPL {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed("compiler> ")
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("\x1b[90m{}\x1b[0m", hint))
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        let mut highlighted = String::new();
        let words: Vec<&str> = line.split_whitespace().collect();

        if let Some(first_word) = words.first() {
            if self.commands.contains_key(*first_word) {
                highlighted.push_str("\x1b[32m");
                highlighted.push_str(first_word);
                highlighted.push_str("\x1b[0m");

                if line.len() > first_word.len() {
                    highlighted.push_str(&line[first_word.len()..]);
                }

                return Owned(highlighted);
            }
        }

        for (i, ch) in line.chars().enumerate() {
            if ch == '(' || ch == ')' || ch == '{' || ch == '}' || ch == '[' || ch == ']' {
                if i == pos || i == pos - 1 {
                    highlighted.push_str("\x1b[1;33m");
                    highlighted.push(ch);
                    highlighted.push_str("\x1b[0m");
                } else {
                    highlighted.push(ch);
                }
            } else {
                highlighted.push(ch);
            }
        }

        Owned(highlighted)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

#[derive(Clone)]
pub struct CompilerValidator;

impl Validator for CompilerValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult> {
        let input = ctx.input();
        let mut stack = Vec::new();

        for ch in input.chars() {
            match ch {
                '(' | '{' | '[' => stack.push(ch),
                ')' => {
                    if stack.pop() != Some('(') {
                        return Ok(ValidationResult::Invalid(Some(
                            "Mismatched parentheses".into(),
                        )));
                    }
                }
                '}' => {
                    if stack.pop() != Some('{') {
                        return Ok(ValidationResult::Invalid(Some("Mismatched braces".into())));
                    }
                }
                ']' => {
                    if stack.pop() != Some('[') {
                        return Ok(ValidationResult::Invalid(Some(
                            "Mismatched brackets".into(),
                        )));
                    }
                }
                _ => {}
            }
        }

        if stack.is_empty() {
            Ok(ValidationResult::Valid(None))
        } else {
            Ok(ValidationResult::Incomplete)
        }
    }
}

impl Validator for CompilerREPL {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult> {
        self.validator.validate(ctx)
    }
}

impl CompilerREPL {
    pub fn new() -> Self {
        let mut commands = HashMap::new();
        for cmd in CompilerCommand::COMMANDS {
            commands.insert(cmd.name.to_string(), cmd.clone());
        }

        let keywords = HashSet::from([
            "fn", "let", "const", "if", "else", "while", "for", "return", "struct", "enum", "impl",
            "trait", "pub", "mod", "use",
        ]);

        Self {
            commands,
            keywords,
            history_file: "compiler_history.txt".to_string(),
            completer: CommandCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter::new(),
            validator: CompilerValidator,
        }
    }
}

impl Default for CompilerREPL {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_editor() -> Result<Editor<CompilerREPL, DefaultHistory>> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();

    let helper = CompilerREPL::new();
    let mut editor = Editor::with_config(config)?;
    editor.set_helper(Some(helper));

    if editor.load_history("compiler_history.txt").is_err() {
        println!("No previous history.");
    }

    Ok(editor)
}

pub fn process_command(line: &str, repl: &CompilerREPL) -> bool {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return true;
    }

    match parts[0] {
        "help" => {
            if parts.len() > 1 {
                if let Some(cmd) = repl.commands.get(parts[1]) {
                    println!("{} - {}", cmd.name, cmd.description);
                    println!("Usage: {} {}", cmd.name, cmd.args);
                } else {
                    println!("Unknown command: {}", parts[1]);
                }
            } else {
                println!("Available commands:");
                for cmd in CompilerCommand::COMMANDS {
                    println!("  {:10} - {}", cmd.name, cmd.description);
                }
            }
        }
        "quit" => return false,
        "load" => println!("Loading file: {:?}", parts.get(1)),
        "compile" => println!("Compiling with options: {:?}", &parts[1..]),
        "run" => println!("Running with arguments: {:?}", &parts[1..]),
        "ast" => println!("Showing AST for: {:?}", parts.get(1)),
        "ir" => println!("Showing IR for: {:?}", parts.get(1)),
        "symbols" => println!("Listing symbols matching: {:?}", parts.get(1)),
        "type" => println!("Type checking: {}", parts[1..].join(" ")),
        _ => println!(
            "Unknown command: {}. Type 'help' for available commands.",
            parts[0]
        ),
    }

    true
}
