use std::process;

use shell;

fn main() {
    match shell::run() {
        Ok(r) => r,
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    };
}
