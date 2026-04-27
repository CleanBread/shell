use std::process::Output;

#[derive(Debug, PartialEq)]
pub(crate) struct ExecuteOutput {
    pub(crate) out: Vec<u8>,
    pub(crate) err: Vec<u8>,
    pub(crate) exit: bool,
}

impl ExecuteOutput {
    pub(crate) const fn new() -> Self {
        Self {
            out: vec![],
            err: vec![],
            exit: false,
        }
    }

    pub(crate) fn err(value: impl AsRef<[u8]>) -> Self {
        Self {
            err: value.as_ref().to_vec(),
            ..ExecuteOutput::new()
        }
    }

    pub(crate) fn exit() -> Self {
        Self {
            exit: true,
            ..ExecuteOutput::new()
        }
    }
}

impl From<String> for ExecuteOutput {
    fn from(value: String) -> Self {
        Self {
            out: (value + "\n").into_bytes(),
            ..Self::new()
        }
    }
}

impl From<&str> for ExecuteOutput {
    fn from(value: &str) -> Self {
        format!("{value}").into()
    }
}

impl From<Option<String>> for ExecuteOutput {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(value) => Self {
                out: value.as_bytes().to_vec(),
                ..Self::new()
            },
            None => Self::new(),
        }
    }
}

impl From<Output> for ExecuteOutput {
    fn from(value: Output) -> Self {
        Self {
            out: value.stdout,
            err: value.stderr,
            ..ExecuteOutput::new()
        }
    }
}
