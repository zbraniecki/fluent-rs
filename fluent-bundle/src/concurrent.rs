use std::sync::Mutex;

use intl_memoizer::{IntlLangMemoizer, Memoizable};
use unic_langid::LanguageIdentifier;

use crate::bundle::FluentBundleBase;
use crate::memoizer::MemoizerKind;
use crate::types::FluentType;

pub type Memoizer = Mutex<IntlLangMemoizer>;

pub type FluentBundle<R> = FluentBundleBase<R, Memoizer>;

impl MemoizerKind for Memoizer {
    fn new(lang: LanguageIdentifier) -> Self
    where
        Self: Sized,
    {
        Mutex::new(IntlLangMemoizer::new(lang))
    }

    fn with_try_get<I, R, U>(&self, args: I::Args, cb: U) -> Result<R, ()>
    where
        Self: Sized,
        I: Memoizable + 'static,
        U: FnOnce(&I) -> R,
    {
        match self.lock() {
            Ok(mut memoizer) => match memoizer.try_get(args) {
                Ok(memoizable) => Ok(cb(&memoizable)),
                Err(_) => Err(()),
            },
            Err(_) => Err(()),
        }
    }

    fn stringify_value(&self, value: &dyn FluentType) -> std::borrow::Cow<'static, str> {
        value.as_string_threadsafe(self)
    }
}
