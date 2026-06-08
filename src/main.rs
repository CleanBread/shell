use dotenvy::dotenv;
use std::process;

use shell;

fn main() {
    dotenv().ok();

    match shell::run() {
        Ok(r) => r,
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    };
}
