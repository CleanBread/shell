use indexmap::IndexMap;
use std::sync::LazyLock;
use std::sync::Mutex;

#[derive(Debug)]
pub(crate) struct Job {
    pub pid: i32,
    pub command: String,
}

#[derive(Debug)]
pub(crate) struct Jobs {
    pub items: IndexMap<u32, Job>,
    pub next_counter: u32,
}

pub(crate) static JOBS: LazyLock<Mutex<Jobs>> = LazyLock::new(|| {
    Mutex::new(Jobs {
        items: IndexMap::new(),
        next_counter: 1,
    })
});
