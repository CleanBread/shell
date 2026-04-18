use std::{env, mem, path::PathBuf, process};

use crate::utils::{CustomError, find_in_path, pritn_error};

pub enum BuiltinCommand {
    ChangeDirectory(Vec<String>),
    Pwd,
    Type(Vec<String>),
    Echo(Vec<String>),
    NotFound(Vec<String>),
    Exit,
}

impl BuiltinCommand {
    fn parse_input(args: &str) -> Vec<String> {
        let mut result = vec![];
        let mut current = String::new();
        let mut chars = args.chars();

        let mut is_backslash = false;

        while let Some(c) = chars.next() {
            if is_backslash {
                current.push(c);
                is_backslash = false;

                continue;
            }

            match c {
                '\'' => {
                    for c in chars.by_ref() {
                        if c == '\'' {
                            break;
                        }

                        current.push(c);
                    }
                }
                '"' => {
                    let mut is_backslash = false;

                    for c in chars.by_ref() {
                        if is_backslash {
                            is_backslash = false;

                            match c {
                                '\\' | '"' | '$' | '`' | '\n' => current.push(c),
                                _ => {
                                    current.push('\\');
                                    current.push(c);
                                }
                            }
                            continue;
                        }

                        match c {
                            '"' => break,
                            '\\' => is_backslash = true,
                            _ => current.push(c),
                        }
                    }
                }

                ' ' => {
                    if !current.is_empty() {
                        result.push(mem::take(&mut current));
                    }
                }
                '\\' => {
                    is_backslash = true;
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if !current.is_empty() {
            result.push(current);
        }

        result
    }

    pub(crate) fn builtin_cd(args: Vec<String>) {
        let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());

        let target = match args.first().map(String::as_str) {
            None | Some("" | "~") => home,
            Some(path) if path.starts_with("~/") => home + &path[1..],
            Some(path) => path.to_string(),
        };

        if let Err(error) = std::env::set_current_dir(&target) {
            eprintln!("{}", error);
        }
    }

    pub(crate) fn builtin_pwd() {
        match env::current_dir() {
            Ok(path) => {
                println!("{}", path.display());
            }
            Err(error) => {
                eprintln!("{}", error);
            }
        }
    }

    pub(crate) fn builtin_type(paths: &[PathBuf], args: Vec<String>) {
        let command_str = args.get(0).unwrap();

        if !matches!(command_str.as_str().into(), BuiltinCommand::NotFound(_)) {
            println!("{}: is a shell builtin", command_str);

            return;
        }

        if let Some(entry) = find_in_path(paths, command_str, Some(0o111)) {
            println!("{} is {}", command_str, entry.path().display());
        } else {
            println!("{}: not found", command_str);
        }
    }

    pub(crate) fn builtin_echo(args: Vec<String>) {
        println!("{}", args.join(" "));
    }

    pub(crate) fn builtin_not_found(paths: &[PathBuf], input: Vec<String>) {
        let (command_str, command_args) = input.split_first().unwrap();

        if let Some(entry) = find_in_path(paths, command_str, Some(0o111)) {
            let path = entry.path();

            if path.is_file() {
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
        let parsed_input = Self::parse_input(input);
        let (command, args) = parsed_input.split_first().unwrap();

        match command.as_str() {
            "cd" => Self::ChangeDirectory(args.to_owned()),
            "pwd" => Self::Pwd,
            "type" => Self::Type(args.to_owned()),
            "echo" => Self::Echo(args.to_owned()),
            "exit" => Self::Exit,
            _ => Self::NotFound(parsed_input),
        }
    }
}

impl From<String> for BuiltinCommand {
    fn from(input: String) -> Self {
        Self::from(input.as_str())
    }
}
