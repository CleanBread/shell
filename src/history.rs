use std::sync::LazyLock;
use std::sync::Mutex;

pub(crate) static HISTORY: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));
