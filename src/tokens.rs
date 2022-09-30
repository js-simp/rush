
/*How to tokenize
first breakdown the command string into independent commands by splitting at separator ';'
Independent commands that finish with '&' run in the background
args are every term that follows separated by a whitespace after the main command
*/


#[derive(Debug, PartialEq)]
pub struct Tokens {
    main_com : String,
    args : Vec<String>,
    in_background : bool
}

impl Tokens {
    fn new(command : &str, in_background : bool) -> Tokens {
        let mut parts = command.split_whitespace();
        let main_com = String::from(parts.next().unwrap());
        let mut args = vec![];
        for arg in parts {
            args.push(String::from(arg));
        }

        Tokens {
            main_com,
            args,
            in_background
        }
    }
}

pub fn tokenize_commands(command_string : &str) -> Vec<Tokens> {
    let mut commands: Vec<Tokens> = vec![];
    for independent_com in command_string.split(';') {
        let mut processes : Vec<&str> = independent_com.trim().split(" & ").collect();
        let foreground = processes.pop(); 
        for background_process in processes {
            if background_process != "" {
                commands.push(Tokens::new(background_process, true));
            }
        }
        match foreground {
            Some(str) => 
            if str.ends_with('&') { 
                let mut chars = str.chars();
                chars.next_back();
                let str_b = chars.as_str();
                commands.push(Tokens::new(str_b, true))
            }
            else if str != "" {commands.push(Tokens::new(str, false))}
            else {}
            ,
            None => ()
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

        assert_eq!(vec![
            Tokens {main_com : String::from("ls"), args : vec![], in_background : false}
            ], tokens);
    }

    #[test]
    fn single_command_with_args() {
        let commands = "ls -a";
        let tokens = tokenize_commands(commands);

        assert_eq!(vec![
            Tokens {main_com : String::from("ls"), args : vec![String::from("-a")], in_background : false}
            ], tokens);
    }

    #[test]
    fn background_process() {
        let commands = "long-running-process &";
        let tokens = tokenize_commands(commands);

        assert_eq!(vec![
            Tokens {main_com : String::from("long-running-process"), args : vec![], in_background : true}
            ], tokens);
    }

    #[test]
    fn background_process_with_other() {
        let commands = "long-running-process & date";
        let tokens = tokenize_commands(commands);

        // TODO this case is not correct
        // it should be `assert_eq!(vec![vec![vec!["long-running-process &"]], vec![vec!["date"]]], tokens);`
        assert_eq!(vec![
            Tokens {
            main_com : String::from("long-running-process"), args: vec![], in_background : true
            }, 
            Tokens {main_com : String::from("date"), args : vec![], in_background: false}
            ], tokens);
    }

    #[test]
    fn semicolon() {
        let commands = "date ; ls";
        let tokens = tokenize_commands(commands);

        assert_eq!(vec![
            Tokens {main_com : String::from("date"), args: vec![], in_background : false}, 
            Tokens {main_com : String::from("ls"), args: vec![], in_background : false}
            ], tokens);
    }

    #[test]
    fn and() {
        let commands = "date && ls";
        let tokens = tokenize_commands(commands);

        assert_eq!(vec![
            Tokens {main_com : String::from("date"), args: vec![String::from("&&"), String::from("ls")], in_background : false}, 
            ], tokens);
    }
}
