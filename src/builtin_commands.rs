use std::{path::PathBuf, process};

use crate::utils::{CustomError, find_in_path, pritn_error};

pub enum BuiltinCommand {
    Type(String),
    Echo(String),
    Exit,
    NotFound(String),
}

impl BuiltinCommand {
    fn parse_input(input: &str) -> (&str, &str) {
        match input.split_once(' ') {
            Some(r) => r,
            None => (input, ""),
        }
    }

    pub(crate) fn builtin_type(paths: &[PathBuf], args: String) {
        let command_str = Self::parse_input(args.as_str()).0;

        if !matches!(command_str.into(), BuiltinCommand::NotFound(_)) {
            Self::builtin_echo(format!("{}: is a shell builtin", command_str));

            return;
        }

        if let Some(entry) = find_in_path(paths, command_str, Some(0o111)) {
            Self::builtin_echo(format!("{} is {}", command_str, entry.path().display()));
        } else {
            Self::builtin_echo(format!("{}: not found", command_str));
        }
    }

    pub(crate) fn builtin_echo(text: String) {
        println!("{}", text);
    }

    pub(crate) fn builtin_not_found(paths: &[PathBuf], input: String) {
        let (command_str, command_args) = Self::parse_input(input.as_str());

        if let Some(entry) = find_in_path(paths, command_str, Some(0o111)) {
            let path = entry.path();

            if path.is_file() {
                let command_args = command_args.split_whitespace();

                match process::Command::new(&path).args(command_args).status() {
                    Ok(status) => {
                        if !status.success() {
                            eprintln!("Command failed for {:?}", path);
                        }
                    }
                    Err(error) => {
                        eprintln!("Command failed: {:?}", error);
                    }
                };
            }
        } else {
            pritn_error(CustomError::CommandNotFound(command_str.to_string()));
        }
    }
}

impl From<&str> for BuiltinCommand {
    fn from(input: &str) -> Self {
        let (command, args) = Self::parse_input(input);

        match command {
            "type" => Self::Type(args.to_owned()),
            "echo" => Self::Echo(args.to_owned()),
            "exit" => Self::Exit,
            _ => Self::NotFound(input.to_owned()),
        }
    }
}
