extern crate libc;
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

fn main() {
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_IGN);
        libc::signal(libc::SIGQUIT, libc::SIG_IGN);
    }
    let mut last_exit_status = true;
    let mut rl = Editor::<()>::new().unwrap();
    let home = env::var("HOME").unwrap();
    if rl.load_history(&format!("{}/.rush_history", home)).is_err() {
        println!("No previous history.");
        File::create(format!("{}/.rush_history", home)).expect("Couldn't create history file");
    }
    loop {
        let prompt_string = generate_prompt(last_exit_status);
        let command_string = read_command(&mut rl, prompt_string);
        let commands = tokenize_commands(&command_string);

        for command in commands {
            last_exit_status = match command.main_com.as_str() {
                "exit" => {
                    rl.save_history(&format!("{}/.rush_history", home))
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
    let mut command_string = rl.readline(&prompt_string).unwrap();

    // this allows for multiline commands
    while command_string.ends_with('\\') {
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
        format!(
            "{}{}{}\u{2ba1}{}  ",
            prompt,
            colors::ANSI_BOLD,
            colors::GREEN,
            colors::RESET
        )
    } else {
        format!(
            "{}{}{}\u{2ba1}{}  ",
            prompt,
            colors::ANSI_BOLD,
            colors::RED,
            colors::RESET
        )
    }
}

fn execute_command(command_tokens: Tokens) -> bool {
    let mut command_instance = Command::new(command_tokens.main_com.as_str());
    let or_com = command_tokens.or_com;
    if let Ok(mut child) = unsafe {
        command_instance.args(command_tokens.args).pre_exec(|| {
            libc::signal(libc::SIGINT, libc::SIG_DFL);
            libc::signal(libc::SIGQUIT, libc::SIG_DFL);
            Result::Ok(())
        })
    }
    .spawn()
    {
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
