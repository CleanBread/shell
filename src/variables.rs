use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

pub(crate) static VARS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) fn expand_vars(args: &mut [String]) {
    let vars = VARS.lock().unwrap();

    for arg in args.iter_mut() {
        if !arg.contains("$") {
            continue;
        }

        let mut result = String::new();
        let mut chars = arg.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '$' => {
                    let mut var = String::new();
                    let with_curly = chars.peek() == Some(&'{');

                    if with_curly {
                        chars.next();
                    }

                    for c in chars.by_ref() {
                        if with_curly && c == '}' {
                            break;
                        } else {
                            var.push(c);
                        }
                    }

                    if let Some(value) = vars.get(&var) {
                        result.push_str(value);
                    } else if let Ok(value) = std::env::var(&var) {
                        result.push_str(&value);
                    }
                }
                _ => {
                    result.push(c);
                }
            }
        }

        *arg = result;
    }
}
