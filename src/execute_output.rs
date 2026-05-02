use std::process::Output;

#[derive(Debug, PartialEq)]
pub(crate) struct ExecuteOutput {
    pub(crate) out: String,
    pub(crate) err: String,
    pub(crate) exit: bool,
}

impl ExecuteOutput {
    pub(crate) const fn new() -> Self {
        Self {
            out: String::new(),
            err: String::new(),
            exit: false,
        }
    }

    pub(crate) fn err(value: String) -> Self {
        Self {
            err: value,
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
            out: value,
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
                out: value,
                ..Self::new()
            },
            None => Self::new(),
        }
    }
}

impl From<Output> for ExecuteOutput {
    fn from(value: Output) -> Self {
        Self {
            out: String::from_utf8_lossy(&value.stdout).into_owned(),
            err: String::from_utf8_lossy(&value.stderr).into_owned(),
            ..ExecuteOutput::new()
        }
    }
}
