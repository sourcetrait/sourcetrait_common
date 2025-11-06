pub trait CommandExt {
    fn with_arg<S: AsRef<std::ffi::OsStr>>(self, arg: S) -> Self;
}

impl CommandExt for std::process::Command {
    fn with_arg<S: AsRef<std::ffi::OsStr>>(mut self, arg: S) -> Self {
        self.arg(arg);
        self
    }
}