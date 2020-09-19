use std::ops::Range;
use std::fmt;

pub trait Slice<'s>: AsRef<str> + Clone + PartialEq + fmt::Display {
    fn slice(&self, range: Range<usize>) -> Self;
    fn trim(&mut self);
    fn as_str(&self) -> &str;
}

impl<'s> Slice<'s> for String {
    fn slice(&self, range: Range<usize>) -> String {
        self[range].to_string()
    }

    fn trim(&mut self) {
        *self = self.trim_end().to_string();
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }
}

impl<'s> Slice<'s> for &'s str {
    fn slice(&self, range: Range<usize>) -> &'s str {
        &self[range]
    }

    fn trim(&mut self) {
        *self = self.trim_end();
    }

    fn as_str(&self) -> &str {
        self
    }
}
