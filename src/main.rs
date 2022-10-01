extern crate rustyline;

use std::env;
use std::fs::File;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

use rustyline::Editor;

mod colors;
mod tokens;

use tokens::tokenize_commands;
use tokens::Tokens;

fn setup() -> (Editor<()>, String) {
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_IGN);
        libc::signal(libc::SIGQUIT, libc::SIG_IGN);
    }
    let mut rl = match Editor::<()>::new() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("Unexpected error creating editor: {e}");
            std::process::exit(1);
        }
    };
    let rush_history = if let Ok(home) = env::var("HOME") {
        format!("{home}/.rush_history")
    } else {
        eprintln!("environment variable HOME not set: qutting");
        std::process::exit(1);
    };

    if rl.load_history(&rush_history).is_err() {
        println!("No previous history.");
        if let Err(e) = File::create(&rush_history) {
            eprintln!("Couldn't create history file: {e}");
            std::process::exit(1);
        }
    }

    (rl, rush_history)
}

fn main() {
    let (mut rl, rush_history) = setup();

    let mut last_exit_status = true;
    loop {
        let prompt_string = generate_prompt(last_exit_status);
        let command_string = read_command(&mut rl, prompt_string);
        let commands = tokenize_commands(&command_string);

        for command in commands {
            // TODO: in the future, more builtins will be desired
            //       exit will be a "special" builtin
            //       but other builtins should not be part of this list
            //       RECOMMEND putting builtin handling inside of
            //       execute_command
            last_exit_status = match command.main_com.as_str() {
                "exit" => {
                    rl.save_history(&rush_history)
                        .expect("Couldn't save history");
                    std::process::exit(0);
                }
                "cd" => change_dir(command.args[0].as_str()),
                _ => execute_command(command),
            };
            if !last_exit_status {
                break;
            }
        }
    }
}

fn read_command(rl: &mut Editor<()>, prompt_string: String) -> String {
    let mut command_string = match rl.readline(&prompt_string) {
        Ok(cs) => cs,
        Err(e) => {
            eprintln!("Unexpected prompt read error: {e}");
            std::process::exit(1);
        }
    };

    // this allows for multiline commands
    while command_string.ends_with('\\') {
        command_string.pop(); // remove the trailing backslash

        let next_string = match rl.readline("") {
            Ok(ns) => ns,
            Err(e) => {
                eprintln!("Unexpected command string read error: {e}");
                std::process::exit(1);
            }
        };
        command_string.push_str(&next_string);
    }

    // add command to history after handling multi-line input
    rl.add_history_entry(&command_string);
    command_string
}

const PROMPT_ICON: &str = "$";
fn generate_prompt(last_exit_status: bool) -> String {
    let path = match env::current_dir() {
        Ok(path) => path
            .to_str()
            .expect("Unexpected stringification of known path")
            .to_owned(),
        Err(e) => {
            format!("Current Directory access failure: {e}")
        }
    };

    let prompt = format!(
        "{}{}{}{}\n",
        colors::ANSI_BOLD,
        colors::ANSI_COLOR_CYAN,
        path,
        colors::RESET
    );

    let color = if last_exit_status {
        colors::GREEN
    } else {
        colors::RED
    };
    format!(
        "{}{}{}{}{} ",
        prompt,
        colors::ANSI_BOLD,
        color,
        PROMPT_ICON,
        colors::RESET
    )
}

fn execute_command(command_tokens: Tokens) -> bool {
    let mut command_instance = Command::new(command_tokens.main_com.as_str());
    let or_com = command_tokens.or_com;

    let child = unsafe {
        command_instance.args(command_tokens.args).pre_exec(|| {
            libc::signal(libc::SIGINT, libc::SIG_DFL);
            libc::signal(libc::SIGQUIT, libc::SIG_DFL);
            Result::Ok(())
        })
    };

    if let Ok(mut child) = child.spawn() {
        if !command_tokens.in_background {
            if child.wait().expect("command wasn't running").success() {
                true
            } else if let Some(token) = or_com {
                execute_command(*token)
            } else {
                // return execute_command(or_com)
                false
            }
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

    if let Err(err) = env::set_current_dir(&new_path) {
        colors::error_logger(format!("Failed to change the directory!\n{}", err));
        false
    } else {
        true
    }
}
