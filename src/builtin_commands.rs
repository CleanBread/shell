use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::utils::{CustomError, pritn_error};

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

        for path in paths {
            let Ok(entries) = path.read_dir() else {
                continue;
            };

            for entry in entries.flatten() {
                if entry.file_name() != command_str {
                    continue;
                }

                let Ok(metadata) = entry.metadata() else {
                    continue;
                };

                let is_executable = metadata.permissions().mode() & 0o111 != 0;

                if !is_executable {
                    continue;
                }

                Self::builtin_echo(format!("{} is {}", command_str, entry.path().display()));

                return;
            }
        }

        Self::builtin_echo(format!("{}: not found", command_str));
    }

    pub(crate) fn builtin_echo(text: String) {
        println!("{}", text);
    }

    pub(crate) fn builtin_not_found(command: String) {
        pritn_error(CustomError::CommandNotFound(command));
    }
}

impl From<&str> for BuiltinCommand {
    fn from(input: &str) -> Self {
        let (command, args) = Self::parse_input(input);

        match command {
            "type" => Self::Type(args.to_owned()),
            "echo" => Self::Echo(args.to_owned()),
            "exit" => Self::Exit,
            _ => Self::NotFound(command.to_owned()),
        }
    }
}
