# shell

> **Educational project** — built to learn how Unix shells work by implementing one from scratch in Rust.

A Unix shell written in Rust with interactive line editing, job control, piping, I/O redirections, variable expansion, and a set of built-in commands.

## Features

### Interactive input
- Raw terminal mode with character-by-character input via `termion`
- Cursor navigation and line editing
- Command history navigation across sessions (persisted via `$HISTFILE`)

### Execution model
- `&&` chaining — run next command only if the previous succeeded
- Pipes (`|`) — connect stdout of one command to stdin of the next
- Background jobs (`&`) — fork and run a pipeline in the background; notified on completion
- SIGCHLD handling via a self-pipe to safely reap child processes from the event loop

### I/O redirection
| Syntax | Effect |
|--------|--------|
| `cmd > file` | Redirect stdout to file (truncate) |
| `cmd >> file` | Append stdout to file |
| `cmd 2> file` | Redirect stderr to file (truncate) |
| `cmd 2>> file` | Append stderr to file |

### Variable expansion
- Shell variables stored in a session-scoped map (separate from environment variables)
- `$VAR` and `${VAR}` syntax
- Falls back to environment variables when a shell variable is not found

### Built-in commands

| Command | Description |
|---------|-------------|
| `cd [dir]` | Change the working directory. `cd`, `cd ~`, and `cd ""` go to `$HOME`; `~/path` is expanded to the home directory. |
| `pwd` | Print the current working directory. |
| `echo [args...]` | Print arguments joined by spaces. `\n` in the string is interpreted as a newline. |
| `exit` | Exit the shell, flushing history to `$HISTFILE`. |
| `type <name>` | Report whether `name` is a shell builtin or an external command, and if external, print its full path. |
| `declare VAR=VALUE` | Set a shell variable. `declare VAR` sets it to an empty string. |
| `declare -p VAR` | Print the value of a shell variable in `declare -- VAR="VALUE"` format. |
| `history [n]` | List command history with line numbers. Optional `n` shows only the last `n` entries. |
| `jobs` | List background jobs with their job number, status (`running`), and command. The most recent job is marked `+`, the previous `-`. |
| `kill <pid>` | Send `SIGTERM` to a process by PID. |
| `kill %<job>` | Send `SIGTERM` to a background job by job number (e.g. `kill %1`). |

## Requirements

- Rust (edition 2024)
- Unix / macOS (relies on `libc`, `fork`, `waitpid`, POSIX signals)

## Build & Run

```sh
cargo build --release
cargo run
```

The shell reads `$HISTFILE` on startup to load history and appends new entries to it on exit.

## Project Structure

| File | Responsibility |
|------|---------------|
| [src/lib.rs](src/lib.rs) | Main event loop, SIGCHLD signal handling, background job reaping |
| [src/command.rs](src/command.rs) | Command parsing (pipes, `&&`, quoting, escaping) and execution dispatch |
| [src/jobs.rs](src/jobs.rs) | Background job registry |
| [src/history.rs](src/history.rs) | Command history with persistence |
| [src/variables.rs](src/variables.rs) | Shell variable storage and `$VAR` / `${VAR}` expansion |
| [src/redirection.rs](src/redirection.rs) | I/O redirection parsing (`>`, `>>`, `2>`, `2>>`) |
| [src/execute_output.rs](src/execute_output.rs) | Buffered output for builtin-to-pipe handoff |
| [src/utils.rs](src/utils.rs) | Input reading, `PATH` resolution, error types |
