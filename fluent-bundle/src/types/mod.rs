mod number;
mod plural;

use crate::memoizer::MemoizerKind;
pub use crate::resolver::scope::Scope;
pub use fluent_syntax::ast;
use intl_pluralrules::{PluralCategory, PluralRuleType};
pub use number::FluentNumber;
use plural::PluralRules;
use std::fmt;
use std::str::FromStr;

pub trait FluentType {
    fn write(
        &self,
        w: &mut impl fmt::Write,
        intls: &intl_memoizer::IntlLangMemoizer,
    ) -> fmt::Result;
}
#[derive(Debug, Clone)]
pub enum FluentValue<S> {
    String(S),
    Number(FluentNumber),
    // Custom(Box<dyn FluentType + Send>),
    // Error(DisplayableNode<'source>),
    // None,
}

impl<S> FluentValue<S> {
    pub fn write<W>(&self, w: &mut W) -> fmt::Result
    where
        W: fmt::Write,
        S: AsRef<str>,
    {
        match self {
            FluentValue::String(s) => w.write_str(s.as_ref()),
            FluentValue::Number(num) => num.write(w),
        }
    }

    pub fn borrowed(&self) -> FluentValue<&str>
    where
        S: AsRef<str>,
    {
        match self {
            FluentValue::String(s) => FluentValue::String(s.as_ref()),
            FluentValue::Number(n) => FluentValue::Number(n.clone()),
        }
    }

    pub fn try_number(v: S) -> Self
    where
        S: ToString,
    {
        let s = v.to_string();
        if let Ok(num) = FluentNumber::from_str(&s.to_string()) {
            FluentValue::Number(num)
        } else {
            FluentValue::String(v)
        }
    }

    pub fn matches<V, R, M, A>(&self, other: &FluentValue<V>, scope: &Scope<R, M, A>) -> bool
    where
        M: MemoizerKind,
        S: AsRef<str>,
        V: AsRef<str>,
    {
        match (self, other) {
            (&FluentValue::String(ref a), &FluentValue::String(ref b)) => a.as_ref() == b.as_ref(),
            (&FluentValue::Number(ref a), &FluentValue::Number(ref b)) => a == b,
            (&FluentValue::String(ref a), &FluentValue::Number(ref b)) => {
                let cat = match a.as_ref() {
                    "zero" => PluralCategory::ZERO,
                    "one" => PluralCategory::ONE,
                    "two" => PluralCategory::TWO,
                    "few" => PluralCategory::FEW,
                    "many" => PluralCategory::MANY,
                    "other" => PluralCategory::OTHER,
                    _ => return false,
                };
                scope
                    .bundle
                    .intls
                    .with_try_get_threadsafe::<PluralRules, _, _>(
                        (PluralRuleType::CARDINAL,),
                        |pr| pr.0.select(b) == Ok(cat),
                    )
                    .unwrap()
            }
            _ => false,
        }
    }
}
