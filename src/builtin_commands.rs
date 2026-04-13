use std::{env, path::PathBuf, process};

use crate::utils::{CustomError, find_in_path, pritn_error};

pub enum BuiltinCommand {
    ChangeDirectory(String),
    Pwd,
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

    pub(crate) fn builtin_cd(args: String) {
        let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let target = match Self::parse_input(&args).0 {
            "" | "~" => home.clone(),
            path => path.replace("~", &home),
        };

        if let Err(error) = std::env::set_current_dir(&target) {
            eprintln!("{}", error);
        }
    }

    pub(crate) fn builtin_pwd() {
        match env::current_dir() {
            Ok(path) => {
                Self::builtin_echo(format!("{}", path.display()));
            }
            Err(error) => {
                eprintln!("{}", error);
            }
        }
    }

    pub(crate) fn builtin_type(paths: &[PathBuf], args: String) {
        let command_str = Self::parse_input(&args).0;

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
        let (command_str, command_args) = Self::parse_input(&input);

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
            "cd" => Self::ChangeDirectory(args.to_owned()),
            "pwd" => Self::Pwd,
            "type" => Self::Type(args.to_owned()),
            "echo" => Self::Echo(args.to_owned()),
            "exit" => Self::Exit,
            _ => Self::NotFound(input.to_owned()),
        }
    }
}
