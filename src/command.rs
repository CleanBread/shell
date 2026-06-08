use anyhow::Result;
use std::{
    env, fs,
    io::{Write, pipe, stderr, stdout},
    mem,
    path::PathBuf,
    process,
};

use crate::{
    execute_output::ExecuteOutput,
    history::HISTORY,
    jobs::JOBS,
    redirection::Redirection,
    utils::{CustomError, find_in_path},
};

type Token = String;
type CommandWithArgs = Vec<Token>;
type Pipeline = Vec<CommandWithArgs>;
type AndChain = Vec<Pipeline>;

pub enum PipeInput {
    Stream(process::ChildStdout),
    Buffer(ExecuteOutput),
}

#[derive(PartialEq)]
pub enum Command {
    History,
    Kill,
    Jobs,
    ChangeDirectory,
    Type,
    Echo,
    Pwd,
    Exit,
    External(String),
}

impl Command {
    pub(crate) fn parse_input(input: &str) -> (AndChain, bool) {
        let is_background_job = input.ends_with(" &");

        let input = if is_background_job {
            &input[..input.len() - 2]
        } else {
            input
        };

        let result = Self::parse_and_chain(input)
            .into_iter()
            .map(|pipeline| {
                Self::parse_pipes(pipeline)
                    .into_iter()
                    .map(Self::parse_command)
                    .collect()
            })
            .collect();

        (result, is_background_job)
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
        format!("{}\r\n", args.join(" ").replace("\\n", "\n")).into()
    }

    pub(crate) fn builtin_jobs() -> ExecuteOutput {
        let jobs = JOBS.lock().unwrap();
        let mut lines = String::new();

        for (index, (job_num, job)) in jobs.items.iter().enumerate() {
            let prefix = if index + 1 == jobs.items.len() {
                "+"
            } else if index + 2 == jobs.items.len() {
                "-"
            } else {
                " "
            };

            lines.push_str(&format!(
                "[{}] {} running    {}\r\n",
                job_num, prefix, job.command
            ));
        }

        lines.into()
    }

    pub(crate) fn builtin_kill(args: &[String]) -> ExecuteOutput {
        let (id, is_job_number) = match args.first() {
            Some(arg) => match arg.strip_prefix('%') {
                Some(val) => (val, true),
                None => (arg.as_str(), false),
            },
            None => return ExecuteOutput::err("kill: no arguments".to_string()),
        };

        let Ok(id) = id.parse::<u32>() else {
            return ExecuteOutput::err(format!("kill: invalid id: {}", id));
        };

        let pid = if is_job_number {
            match JOBS.lock().unwrap().items.get(&id) {
                Some(job) => job.pid,
                None => return ExecuteOutput::err(format!("kill: %{}: no such job", id)),
            }
        } else {
            id as i32
        };

        let result = unsafe { libc::kill(pid, libc::SIGTERM) };
        if result == -1 {
            return ExecuteOutput::err(format!(
                "kill: {}: {}",
                pid,
                std::io::Error::last_os_error()
            ));
        }

        ExecuteOutput::new()
    }

    pub(crate) fn builtin_history(args: &[String]) -> ExecuteOutput {
        let history = &HISTORY.lock().unwrap().items;

        let skip = args
            .first()
            .and_then(|a| a.parse::<usize>().ok())
            .map(|n| history.len().saturating_sub(n))
            .unwrap_or(0);

        let lines = history
            .iter()
            .enumerate()
            .skip(skip)
            .map(|(i, cmd)| format!("{:>6} {}\r\n", i + 1, cmd))
            .collect::<String>();

        lines.into()
    }

    pub(crate) fn execute(
        paths: &[PathBuf],
        and_chain: AndChain,
        is_background: bool,
    ) -> Result<bool> {
        for pipeline in and_chain {
            let mut children: Vec<process::Child> = vec![];
            let mut pipe_input: Option<PipeInput> = None;

            for command in pipeline {
                let (command, args) = command.split_first().unwrap();

                let command: Command = command.as_str().into();

                let (args, redirection) = Redirection::extract(args);

                match command {
                    Command::External(external) => {
                        let stdin = match pipe_input.take() {
                            Some(input) => match input {
                                PipeInput::Stream(child_stdout) => {
                                    process::Stdio::from(child_stdout)
                                }
                                PipeInput::Buffer(buffer) => {
                                    let (reader, mut writer) = pipe()?;

                                    std::thread::spawn(move || writer.write(buffer.out.as_bytes()));

                                    process::Stdio::from(reader)
                                }
                            },
                            None => process::Stdio::inherit(),
                        };

                        let (stdout, stderr) = match redirection {
                            Some(redirection) => match redirection {
                                Redirection::RedirectStdout(path) => {
                                    let file = fs::File::create(path)?;

                                    (process::Stdio::from(file), process::Stdio::inherit())
                                }
                                Redirection::RedirectStderr(path) => {
                                    let file = fs::File::create(path)?;

                                    (process::Stdio::piped(), process::Stdio::from(file))
                                }
                                Redirection::AppendStdout(path) => {
                                    let file = fs::OpenOptions::new()
                                        .append(true)
                                        .create(true)
                                        .open(path)?;

                                    (process::Stdio::from(file), process::Stdio::inherit())
                                }
                                Redirection::AppendStderr(path) => {
                                    let file = fs::OpenOptions::new()
                                        .append(true)
                                        .create(true)
                                        .open(path)?;

                                    (process::Stdio::piped(), process::Stdio::from(file))
                                }
                            },
                            None => (process::Stdio::piped(), process::Stdio::inherit()),
                        };

                        match Command::execute_external(
                            &paths, &external, args, stdin, stdout, stderr,
                        ) {
                            Ok(mut child) => {
                                pipe_input = match child.stdout.take() {
                                    Some(out) => Some(PipeInput::Stream(out)),
                                    None => Some(PipeInput::Buffer(ExecuteOutput::new())),
                                };

                                children.push(child);
                            }
                            Err(error) => {
                                pipe_input =
                                    Some(PipeInput::Buffer(ExecuteOutput::err(error.to_string())));

                                break;
                            }
                        }
                    }
                    _ => {
                        let output = command.execute_builtin(&args, &paths);

                        if let Some(redirection) = redirection {
                            match redirection {
                                Redirection::RedirectStdout(path) => {
                                    fs::write(path, output.out).ok();
                                }
                                Redirection::RedirectStderr(path) => {
                                    fs::write(path, output.err).ok();
                                }
                                Redirection::AppendStdout(path) => {
                                    let file =
                                        fs::OpenOptions::new().append(true).create(true).open(path);

                                    if let Result::Ok(mut file) = file {
                                        file.write_all(output.out.as_bytes()).ok();
                                    }
                                }
                                Redirection::AppendStderr(path) => {
                                    let file =
                                        fs::OpenOptions::new().append(true).create(true).open(path);

                                    if let Result::Ok(mut file) = file {
                                        file.write_all(output.err.as_bytes()).ok();
                                    }
                                }
                            };
                        } else {
                            pipe_input = Some(PipeInput::Buffer(output))
                        }
                    }
                };
            }

            for mut child in children {
                child.wait()?;
            }

            match pipe_input {
                Some(PipeInput::Stream(mut stream)) => {
                    // stdout().suspend_raw_mode().unwrap();
                    if is_background {
                        write!(stdout(), "\r\n")?;
                    }
                    std::io::copy(&mut stream, &mut stdout())?;
                    // stdout().activate_raw_mode().unwrap();
                }
                Some(PipeInput::Buffer(output)) => {
                    if output.exit {
                        return Ok(false);
                    }

                    if !output.out.is_empty() {
                        if is_background {
                            write!(stdout(), "\r\n")?;
                        }
                        write!(stdout(), "{}", output.out)?;
                    }

                    if !output.err.is_empty() {
                        if is_background {
                            write!(stderr(), "\r\n")?;
                        }
                        write!(stderr(), "{}", output.err)?;

                        break;
                    }
                }
                _ => {}
            }
        }

        Ok(true)
    }

    pub(crate) fn execute_external(
        paths: &[PathBuf],
        command: &str,
        args: &[String],
        stdin: process::Stdio,
        stdout: process::Stdio,
        stderr: process::Stdio,
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
            Command::History => Command::builtin_history(args),
            Command::Kill => Command::builtin_kill(args),
            Command::Jobs => Command::builtin_jobs(),
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
            "history" => Command::History,
            "kill" => Command::Kill,
            "jobs" => Command::Jobs,
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
