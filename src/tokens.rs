/*
 * How to tokenize
 * 1) breakdown the command string into independent commands
 * by splitting at separator ';'
 * 2) breakdown the independent commands into dependent_commands
 * by splitting at separator "&&"
 * 3) breakdown background commands
 * by splitting at separator " & "
 *
 * Independent commands that finish with '&' run in the background
 * args are every term that follows
 * separated by a whitespace after the main command
*/

#[derive(Debug, PartialEq)]
pub struct Tokens {
    pub main_com: String,
    pub args: Vec<String>,
    pub or_com: Option<Box<Tokens>>,
    pub in_background: bool,
}

impl Tokens {
    // TODO: what if command is_empty() ?
    fn new(command: &str, in_background: bool) -> Tokens {
        let dep_coms = command.split_once("||");

        let (initial_com, or_com) = if let Some(dep_com) = dep_coms {
            let tok = Some(Box::new(Tokens::new(dep_com.1, in_background)));
            (dep_com.0, tok)
        } else {
            (command, None)
        };

        let mut parts = initial_com.split_whitespace();
        let main_com = String::from(parts.next().expect("Unexpected empty parts"));

        let mut args = vec![];
        for arg in parts {
            args.push(String::from(arg));
        }

        Tokens {
            main_com,
            args,
            or_com,
            in_background,
        }
    }
}

pub fn tokenize_commands(command_string: &str) -> Vec<Tokens> {
    let mut commands = vec![];

    for independent_com in command_string.split(';') {
        for dependent_coms in independent_com.split("&&") {
            let mut processes: Vec<&str> = dependent_coms.trim().split(" & ").collect();
            let foreground = processes.pop();

            for background_process in processes {
                if !background_process.is_empty() {
                    commands.push(Tokens::new(background_process, true));
                }
            }

            if let Some(s) = foreground {
                if s.ends_with('&') {
                    let mut chars = s.chars();
                    chars.next_back();
                    let str_b = chars.as_str();
                    commands.push(Tokens::new(str_b, true));
                } else if !s.is_empty() {
                    commands.push(Tokens::new(s, false));
                }
            }
        }
    }
    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_command() {
        let commands = "ls";
        let tokens = tokenize_commands(commands);

        assert_eq!(
            vec![Tokens {
                main_com: String::from("ls"),
                args: vec![],
                or_com: None,
                in_background: false
            }],
            tokens
        );
    }

    #[test]
    fn single_command_with_args() {
        let commands = "ls -a";
        let tokens = tokenize_commands(commands);

        assert_eq!(
            vec![Tokens {
                main_com: String::from("ls"),
                args: vec![String::from("-a")],
                or_com: None,
                in_background: false
            }],
            tokens
        );
    }

    #[test]
    fn background_process() {
        let commands = "long-running-process &";
        let tokens = tokenize_commands(commands);

        assert_eq!(
            vec![Tokens {
                main_com: String::from("long-running-process"),
                args: vec![],
                or_com: None,
                in_background: true
            }],
            tokens
        );
    }

    #[test]
    fn background_process_with_other() {
        let commands = "long-running-process & date";
        let tokens = tokenize_commands(commands);

        // TODO this case is not correct
        // it should be `assert_eq!(vec![vec![vec!["long-running-process &"]], vec![vec!["date"]]], tokens);`
        assert_eq!(
            vec![
                Tokens {
                    main_com: String::from("long-running-process"),
                    args: vec![],
                    or_com: None,
                    in_background: true
                },
                Tokens {
                    main_com: String::from("date"),
                    args: vec![],
                    or_com: None,
                    in_background: false
                }
            ],
            tokens
        );
    }

    #[test]
    fn semicolon() {
        let commands = "date ; ls";
        let tokens = tokenize_commands(commands);

        assert_eq!(
            vec![
                Tokens {
                    main_com: String::from("date"),
                    args: vec![],
                    or_com: None,
                    in_background: false
                },
                Tokens {
                    main_com: String::from("ls"),
                    args: vec![],
                    or_com: None,
                    in_background: false
                }
            ],
            tokens
        );
    }

    #[test]
    fn and() {
        let commands = "date && ls";
        let tokens = tokenize_commands(commands);

        assert_eq!(
            vec![
                Tokens {
                    main_com: String::from("date"),
                    args: vec![],
                    or_com: None,
                    in_background: false
                },
                Tokens {
                    main_com: String::from("ls"),
                    args: vec![],
                    or_com: None,
                    in_background: false
                }
            ],
            tokens
        );
    }

    #[test]
    fn and_or() {
        let commands = "date && ls || ls";
        let tokens = tokenize_commands(commands);

        assert_eq!(
            vec![
                Tokens {
                    main_com: String::from("date"),
                    args: vec![],
                    or_com: None,
                    in_background: false
                },
                Tokens {
                    main_com: String::from("ls"),
                    args: vec![],
                    or_com: Some(Box::new(Tokens {
                        main_com: String::from("ls"),
                        args: vec![],
                        or_com: None,
                        in_background: false
                    })),
                    in_background: false
                }
            ],
            tokens
        );
    }
}
