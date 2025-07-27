use crate::*;

pub trait AsStr {
    fn as_str(&self) -> &str;
}

impl<T: Into<&'static str> + AsRef<str>> AsStr for T {
    fn as_str(&self) -> &str { self.as_ref() }
}
