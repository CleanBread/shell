use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

pub(crate) static VARS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
