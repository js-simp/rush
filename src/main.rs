extern crate libc;
extern crate rustyline;
extern crate rustyline_derive;


use std::borrow::Cow::{self, Borrowed, Owned};

use std::env;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::fs::File;

use rustyline::Editor;
use rustyline::hint::HistoryHinter;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline_derive::{Helper, Hinter, Validator, Completer};

mod colors;
mod tokens;

use tokens::tokenize_commands;
use tokens::Tokens;

#[derive(Helper, Hinter, Validator, Completer)]
struct MyHelper {
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
}
impl Highlighter for MyHelper {
    
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

fn main() {
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_IGN);
        libc::signal(libc::SIGQUIT, libc::SIG_IGN);
    }
    let h = MyHelper {
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
    };
    let mut last_exit_status = true;
    let mut rl = Editor::<MyHelper>::new().unwrap();
    rl.set_helper(Some(h));
    let home = env::var("HOME").unwrap();
    if rl.load_history(&format!("{}/.rush_history", home)).is_err() {
        println!("No previous history.");
        File::create(format!("{}/.rush_history", home)).expect("Couldn't create history file");
    }
    loop {
        let prompt_string = generate_prompt(last_exit_status);
        let command_string = read_command(&mut rl, prompt_string);
        let commands = tokenize_commands(&command_string);

        for mut command in commands {
            last_exit_status = true;
            match command.main_com.as_str() {
                "exit" => {
                    rl.save_history(&format!("{}/.rush_history", home)).expect("Couldn't save history");
                    std::process::exit(0);
                },
                "cd" => {
                    last_exit_status = change_dir(command.args[0].as_str());
                },
                _ => {
                    last_exit_status = execute_command(command);
                }
            }
            if last_exit_status == false {
                break;
            }
        }
    }
}

fn read_command(rl: &mut Editor<MyHelper>, prompt_string: String) -> String {
    let mut command_string = rl.readline(&prompt_string).unwrap();

    // this allows for multiline commands
    while command_string.chars().last() == Some('\\') {
        command_string.pop(); // remove the trailing backslash
        let next_string = rl.readline("").unwrap();
        command_string.push_str(&next_string);
    }

    // add command to history after handling multi-line input
    rl.add_history_entry(&command_string);
    command_string
}

fn generate_prompt(last_exit_status: bool) -> String {
    let path = env::current_dir().unwrap();
    let prompt = format!(
        "{}RUSHING IN {}{}{}\n",
        colors::ANSI_BOLD,
        colors::ANSI_COLOR_CYAN,
        path.display(),
        colors::RESET
    );
    if last_exit_status {
        return format!(
            "{}{}{}\u{2ba1}{}  ",
            prompt,
            colors::ANSI_BOLD,
            colors::GREEN,
            colors::RESET
        );
    } else {
        return format!(
            "{}{}{}\u{2ba1}{}  ",
            prompt,
            colors::ANSI_BOLD,
            colors::RED,
            colors::RESET
        );
    }
}

fn execute_command(command_tokens: Tokens) -> bool {
    let mut command_instance = Command::new(command_tokens.main_com.as_str());
    let or_com = command_tokens.or_com;
    if let Ok(mut child) = command_instance
        .args(command_tokens.args)
        .before_exec(|| {
            unsafe {
                libc::signal(libc::SIGINT, libc::SIG_DFL);
                libc::signal(libc::SIGQUIT, libc::SIG_DFL);
            }
            Result::Ok(())
        })
        .spawn()
    {
        if command_tokens.in_background == false {
            if child.wait().expect("command wasn't running").success() {
                return true
            }
            else {
                if or_com != None {
                    match or_com {
                        Some(token) => { return execute_command(*token)},
                        None => return false
                    }
                    // return execute_command(or_com)
                }
                else {
                    return false
                }
            };
        } else {
            colors::success_logger(format!("{} started!", child.id()));
            true
        }
    } else {
        colors::error_logger("Command not found!".to_string());
        false
    }
}

fn change_dir(new_path: &str) -> bool {
    let new_path = Path::new(new_path);
    match env::set_current_dir(&new_path) {
        Err(err) => {
            colors::error_logger(format!("Failed to change the directory!\n{}", err));
            return false;
        }
        _ => (),
    }
    return true;
}
