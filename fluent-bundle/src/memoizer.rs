use intl_memoizer::{IntlLangMemoizer, Memoizable};
use std::fmt;
use unic_langid::LanguageIdentifier;

pub trait MemoizerKind: 'static {
    fn new(lang: LanguageIdentifier) -> Self
    where
        Self: Sized;

    fn with_try_get_threadsafe<I, R, U>(&self, args: I::Args, cb: U) -> Result<R, I::Error>
    where
        Self: Sized,
        I: Memoizable + Send + Sync + 'static,
        I::Args: Send + Sync + 'static,
        U: FnOnce(&I) -> R;

    fn write<W>(&self, w: &mut W) -> fmt::Result
    where
        W: fmt::Write;
}

impl MemoizerKind for IntlLangMemoizer {
    fn new(lang: LanguageIdentifier) -> Self
    where
        Self: Sized,
    {
        IntlLangMemoizer::new(lang)
    }

    fn with_try_get_threadsafe<I, R, U>(&self, args: I::Args, cb: U) -> Result<R, I::Error>
    where
        Self: Sized,
        I: Memoizable + Send + Sync + 'static,
        I::Args: Send + Sync + 'static,
        U: FnOnce(&I) -> R,
    {
        self.with_try_get(args, cb)
    }

    fn write<W>(&self, w: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        panic!();
    }
}
