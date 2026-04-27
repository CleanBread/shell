use std::{env, fs, io::Write, mem, path::PathBuf, process};

use crate::{
    execute_output::ExecuteOutput,
    utils::{CustomError, find_in_path},
};

#[derive(PartialEq)]
pub enum BuiltinCommand {
    ChangeDirectory,
    Type,
    Echo,
    NotFound(String),
    Pwd,
    Exit,
}

impl BuiltinCommand {
    pub(crate) fn parse_input(args: &str) -> Vec<String> {
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

    pub(crate) fn builtin_cd(args: &[String]) -> ExecuteOutput {
        let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());

        let target = match args.first().map(String::as_str) {
            None | Some("" | "~") => home,
            Some(path) if path.starts_with("~/") => home + &path[1..],
            Some(path) => path.to_string(),
        };

        if let Err(error) = std::env::set_current_dir(&target) {
            return ExecuteOutput::err(format!("{}", error));
        }

        ExecuteOutput::new()
    }

    pub(crate) fn builtin_pwd() -> ExecuteOutput {
        match env::current_dir() {
            Ok(path) => format!("{}", path.display()).into(),
            Err(error) => ExecuteOutput::err(format!("{}", error)),
        }
    }

    pub(crate) fn builtin_type(paths: &[PathBuf], args: &[String]) -> ExecuteOutput {
        let command_str = args.get(0).unwrap();
        let command: BuiltinCommand = command_str.clone().into();

        if !matches!(command, BuiltinCommand::NotFound(_)) {
            return format!("{}: is a shell builtin", command_str).into();
        }

        if let Some(entry) = find_in_path(paths, command_str, Some(0o111)) {
            return format!("{} is {}", command_str, entry.path().display()).into();
        }

        format!("{}: not found", command_str).into()
    }

    pub(crate) fn builtin_echo(args: &[String]) -> ExecuteOutput {
        format!("{}", args.join(" ")).into()
    }

    pub(crate) fn builtin_not_found(
        paths: &[PathBuf],
        command_str: &str,
        command_args: &[String],
    ) -> ExecuteOutput {
        if let Some(entry) = find_in_path(paths, command_str, Some(0o111)) {
            let path = entry.path();

            if path.is_file() {
                let mut command = process::Command::new(&path);
                command.args(command_args);

                match command.output() {
                    Ok(output) => {
                        if !output.status.success() {
                            return ExecuteOutput::err(format!("Command failed for {:?}", path));
                        }

                        return output.into();
                    }
                    Err(error) => {
                        return ExecuteOutput::err(format!("Command failed: {:?}", error));
                    }
                };
            }
        }

        ExecuteOutput::err(CustomError::CommandNotFound(command_str.to_string()).to_string())
    }

    pub(crate) fn execute(&self, args: &[String], paths: &[PathBuf]) -> ExecuteOutput {
        match args {
            [head @ .., arg, path] if arg == ">" || arg == "1>" => {
                let output = self.execute(head, paths);

                fs::write(path, output.out).ok();

                return ExecuteOutput::new();
            }
            [head @ .., arg, path] if arg == ">>" || arg == "1>>" => {
                let output = self.execute(head, paths);

                let file = fs::OpenOptions::new().append(true).create(true).open(path);

                if let Result::Ok(mut file) = file {
                    file.write_all(&output.out).ok();
                }

                return ExecuteOutput::new();
            }
            [head @ .., arg, path] if arg == "2>" => {
                let output = self.execute(head, paths);

                fs::write(path, output.err).ok();

                return ExecuteOutput::new();
            }
            [head @ .., arg, path] if arg == "2>>" => {
                let output = self.execute(head, paths);

                let file = fs::OpenOptions::new().append(true).create(true).open(path);

                if let Result::Ok(mut file) = file {
                    file.write_all(&output.err).ok();
                }

                return ExecuteOutput::new();
            }
            _ => (),
        }

        match self {
            BuiltinCommand::ChangeDirectory => BuiltinCommand::builtin_cd(args),
            BuiltinCommand::Pwd => BuiltinCommand::builtin_pwd(),
            BuiltinCommand::Type => BuiltinCommand::builtin_type(&paths, args),
            BuiltinCommand::Echo => BuiltinCommand::builtin_echo(args),
            BuiltinCommand::NotFound(command) => {
                BuiltinCommand::builtin_not_found(&paths, command, args)
            }
            BuiltinCommand::Exit => ExecuteOutput::exit(),
        }
    }
}

impl From<&str> for BuiltinCommand {
    fn from(command: &str) -> Self {
        match command {
            "cd" => BuiltinCommand::ChangeDirectory,
            "pwd" => BuiltinCommand::Pwd,
            "type" => BuiltinCommand::Type,
            "echo" => BuiltinCommand::Echo,
            "exit" => BuiltinCommand::Exit,
            _ => BuiltinCommand::NotFound(command.to_owned()),
        }
    }
}

impl From<String> for BuiltinCommand {
    fn from(input: String) -> Self {
        Self::from(input.as_str())
    }
}
