use std::{
    env,
    error::Error,
    io::{Write, stdin, stdout},
    path::Path,
    process::{Child, Command, Stdio},
};

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        print!("> ");
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Split input on pipe characters to handle command chaining
        let mut commands = input.trim().split(" | ").peekable();
        let mut prev_stdout = None;
        let mut children: Vec<Child> = Vec::new();

        // Process each command in the pipeline
        while let Some(command) = commands.next() {
            let mut parts = command.split_whitespace();
            let Some(command) = parts.next() else {
                continue;
            };
            let args = parts;

            match command {
                "cd" => {
                    // Built-in: change directory
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(root) {
                        eprintln!("cd: {}", e);
                    }
                    // Reset prev_stdout since cd doesn't produce output
                    prev_stdout = None;
                }
                "exit" => {
                    println!("Goodbye!");
                    return Ok(());
                }
                command => {
                    // External command: set up stdin/stdout for piping

                    // Input: either from previous command's output or inherit from shell
                    let stdin = match prev_stdout.take() {
                        Some(output) => Stdio::from(output),
                        None => Stdio::inherit(),
                    };

                    // Output: pipe to next command if there is one, otherwise inherit
                    let stdout = if commands.peek().is_some() {
                        Stdio::piped() // More commands follow, so pipe output
                    } else {
                        Stdio::inherit() // Last command, output to terminal
                    };

                    // Spawn the command with configured stdin/stdout
                    let child = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match child {
                        Ok(mut child) => {
                            // Take ownership of stdout for next command in pipeline
                            prev_stdout = child.stdout.take();
                            children.push(child);
                        }
                        Err(e) => {
                            eprintln!("Failed to execute '{}': {}", command, e);
                            break;
                        }
                    }
                }
            }
        }

        // Wait for all child processes to complete
        for mut child in children {
            let _ = child.wait();
        }
    }
}
