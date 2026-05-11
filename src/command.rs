use anyhow::Result;
use std::{env, mem, path::PathBuf, process};

use crate::{
    execute_output::ExecuteOutput,
    utils::{CustomError, find_in_path},
};

type Token = String;
type CommandWithArgs = Vec<Token>;
type Pipeline = Vec<CommandWithArgs>;
type AndChain = Vec<Pipeline>;

#[derive(PartialEq)]
pub enum Command {
    ChangeDirectory,
    Type,
    Echo,
    Pwd,
    Exit,
    External(String),
}

impl Command {
    pub(crate) fn parse_input(input: &str) -> AndChain {
        Self::parse_and_chain(input)
            .into_iter()
            .map(|pipeline| {
                Self::parse_pipes(pipeline)
                    .into_iter()
                    .map(Self::parse_command)
                    .collect()
            })
            .collect()
    }

    fn parse_pipes(input: &str) -> Vec<&str> {
        input.split(" | ").collect()
    }

    fn parse_and_chain(args: &str) -> Vec<&str> {
        args.split(" && ").collect()
    }

    fn parse_command(command: &str) -> Vec<String> {
        let mut result = vec![];
        let mut current = String::new();
        let mut chars = command.chars();
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
                '\\' => is_backslash = true,
                _ => current.push(c),
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
        let Some(command_str) = args.get(0) else {
            return ExecuteOutput::new();
        };

        let command: Command = command_str.clone().into();

        if !matches!(command, Command::External(_)) {
            return format!("{}: is a shell builtin", command_str).into();
        }

        if let Some(entry) = find_in_path(paths, command_str, Some(0o111)) {
            return format!("{} is {}", command_str, entry.path().display()).into();
        }

        format!("{}: not found", command_str).into()
    }

    pub(crate) fn builtin_echo(args: &[String]) -> ExecuteOutput {
        format!("{}\n", args.join(" ").replace("\\n", "\n")).into()
    }

    pub(crate) fn execute_external(
        paths: &[PathBuf],
        command: &str,
        args: &[String],
        stdin: process::Stdio,
        stdout: process::Stdio,
        stderr: process::Stdio,
        // input: Option<PipeInput>,
    ) -> Result<process::Child> {
        if let Some(entry) = find_in_path(paths, command, Some(0o111)) {
            let path = entry.path();

            if path.is_file() {
                let child = process::Command::new(&path)
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .stderr(stderr)
                    .spawn()?;

                return Ok(child);
            }
        }

        Err(CustomError::CommandNotFound(command.to_string()).into())
    }

    pub(crate) fn execute_builtin(&self, args: &[String], paths: &[PathBuf]) -> ExecuteOutput {
        match self {
            Command::ChangeDirectory => Command::builtin_cd(args),
            Command::Pwd => Command::builtin_pwd(),
            Command::Type => Command::builtin_type(&paths, args),
            Command::Echo => Command::builtin_echo(args),
            Command::Exit => ExecuteOutput::exit(),
            _ => ExecuteOutput::new(),
        }
    }
}

impl From<&str> for Command {
    fn from(command: &str) -> Self {
        match command {
            "cd" => Command::ChangeDirectory,
            "pwd" => Command::Pwd,
            "type" => Command::Type,
            "echo" => Command::Echo,
            "exit" => Command::Exit,
            _ => Command::External(command.to_owned()),
        }
    }
}

impl From<String> for Command {
    fn from(input: String) -> Self {
        Self::from(input.as_str())
    }
}
