pub enum Redirection<'a> {
    RedirectStdout(&'a str),
    RedirectStderr(&'a str),
    AppendStdout(&'a str),
    AppendStderr(&'a str),
}

impl<'a> Redirection<'a> {
    pub(crate) fn extract(args: &'a [String]) -> (&'a [String], Option<Self>) {
        match args {
            [head @ .., arg, path] if arg == ">" || arg == "1>" => {
                (head, Some(Redirection::RedirectStdout(path)))
            }
            [head @ .., arg, path] if arg == ">>" || arg == "1>>" => {
                (head, Some(Redirection::AppendStdout(path)))
            }
            [head @ .., arg, path] if arg == "2>" => {
                (head, Some(Redirection::RedirectStderr(path)))
            }
            [head @ .., arg, path] if arg == "2>>" => (head, Some(Redirection::AppendStderr(path))),
            _ => (args, None),
        }
    }
}
