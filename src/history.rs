use std::io::BufWriter;
use std::io::Write;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::{env, fs};

pub struct History {
    pub(crate) items: Vec<String>,
    file_path: String,
    stored_index: usize,
}

pub(crate) static HISTORY: LazyLock<Mutex<History>> = LazyLock::new(|| {
    let file_path = match env::var("HISTFILE") {
        Ok(path) => path,
        Err(e) => panic!("Couldn't read HISTFILE: {e}"),
    };

    let stored = fs::read_to_string(&file_path).unwrap_or(String::new());

    let items: Vec<String> = stored.lines().map(String::from).collect();
    let stored_index = items.len();

    Mutex::new(History {
        items,
        file_path,
        stored_index,
    })
});

impl History {
    pub(crate) fn store() {
        let history = HISTORY.lock().unwrap();

        let file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&history.file_path)
            .expect("Couldn't read HISTFILE: {history.file_path}");

        let mut writer = BufWriter::new(file);
        let items_to_store = &history.items[history.stored_index..];

        for item in items_to_store {
            writeln!(writer, "{}", item).expect("Couldn't store to HISTFILE");
        }
    }
}
