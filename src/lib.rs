use anyhow::Result;
use os_pipe::pipe;
use std::{
    fs,
    io::{Write, stderr, stdout},
    process,
};
use termion::{
    cursor::{self},
    raw::IntoRawMode,
};

use crate::{
    command::Command,
    execute_output::ExecuteOutput,
    redirection::Redirection,
    utils::{PipeInput, get_input, get_paths},
};

mod command;
mod execute_output;
mod redirection;
mod test;
mod utils;

pub fn run() -> Result<()> {
    let mut stdout = stdout().lock().into_raw_mode().unwrap();
    let mut stderr = stderr().lock().into_raw_mode().unwrap();

    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        cursor::BlinkingUnderline,
    )
    .unwrap();
    stdout.flush().unwrap();

    let paths = get_paths()?;

    loop {
        let input = get_input()?;

        if input.is_empty() {
            continue;
        }

        let and_chain = Command::parse_input(&input);

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

                                    std::thread::spawn(move || {
                                        writer.write_all(buffer.out.as_bytes())
                                    });

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
                    stdout.suspend_raw_mode().unwrap();
                    std::io::copy(&mut stream, &mut std::io::stdout())?;
                    stdout.activate_raw_mode().unwrap();
                }
                Some(PipeInput::Buffer(output)) => {
                    if output.exit {
                        stdout.suspend_raw_mode().unwrap();
                        stderr.suspend_raw_mode().unwrap();

                        return Ok(());
                    }

                    if !output.out.is_empty() {
                        write!(stdout, "{}", output.out)?;
                        stdout.flush()?;
                    }

                    if !output.err.is_empty() {
                        write!(stderr, "{}", output.err)?;
                        stderr.flush()?;

                        break;
                    }
                }
                _ => {}
            }
        }
    }
}
