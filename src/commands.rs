pub enum Command {
    Echo(String),
    Exit,
    NotFound(String),
}

impl Command {
    fn parse_input(input: &str) -> (&str, &str) {
        match input.split_once(' ') {
            Some(r) => r,
            None => (input, ""),
        }
    }
}

impl From<&str> for Command {
    fn from(input: &str) -> Self {
        let (command, args) = Self::parse_input(input);

        match command {
            "type" => {
                let sub_command = Self::parse_input(args).0;

                match sub_command.into() {
                    Command::NotFound(c) => Self::Echo(format!("{}: not found", c)),
                    _ => Self::Echo(format!("{}: is a shell builtin", sub_command)),
                }
            }
            "echo" => Self::Echo(args.to_owned()),
            "exit" => Self::Exit,
            _ => Self::NotFound(command.to_owned()),
        }
    }
}
