use crate::types::FluentType;
use intl_memoizer::Memoizable;
use unic_langid::LanguageIdentifier;

pub trait MemoizerKind: 'static {
    fn new(lang: LanguageIdentifier) -> Self
    where
        Self: Sized;

    fn with_try_get<I, R, U>(&self, args: I::Args, cb: U) -> Result<R, ()>
    where
        Self: Sized,
        I: Memoizable + 'static,
        U: FnOnce(&I) -> R;

    fn stringify_value(&self, value: &dyn FluentType) -> std::borrow::Cow<'static, str>;
}
