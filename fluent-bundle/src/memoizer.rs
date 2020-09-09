pub trait MemoizerKind: 'static {
    fn new() -> Self
    where
        Self: Sized;
}

pub struct Memoizer {}

impl Memoizer {}

impl MemoizerKind for Memoizer {
    fn new() -> Self {
        Self {}
    }
}
