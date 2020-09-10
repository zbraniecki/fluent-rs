use crate::args::FluentArgs;
use crate::errors::FluentError;
use crate::memoizer::MemoizerKind;
use crate::resolver::{scope::Scope, ResolveValue};
use crate::resource::FluentResource;
use fluent_syntax::ast;
use intl_memoizer::IntlLangMemoizer;
use std::borrow::Borrow;
use std::fmt;
use unic_langid::LanguageIdentifier;

pub struct FluentBundle<R, M> {
    resources: Vec<R>,
    pub intls: M,
}

impl<R> FluentBundle<R, IntlLangMemoizer> {
    pub fn new() -> Self {
        FluentBundle {
            resources: vec![],
            intls: IntlLangMemoizer::new(LanguageIdentifier::default()),
        }
    }
}

impl<R, M> FluentBundle<R, M> {
    pub fn new_with_memoizer(intls: M) -> Self {
        FluentBundle {
            resources: vec![],
            intls,
        }
    }

    pub fn add_resource(&mut self, r: R) -> Result<(), Vec<FluentError>>
    where
        R: Borrow<FluentResource>,
    {
        self.resources.push(r);
        Ok(())
    }

    pub fn get_message(&self, id: &str) -> Option<ast::Message<&str>>
    where
        R: Borrow<FluentResource>,
    {
        for res in &self.resources {
            let res = res.borrow();
            if let Some(msg) = res.get_message(id) {
                return Some(msg);
            }
        }
        None
    }

    pub fn format_pattern<'bundle, S, A, W: fmt::Write>(
        &'bundle self,
        w: &mut W,
        pattern: &ast::Pattern<S>,
        args: Option<FluentArgs<A>>,
    ) -> fmt::Result
    where
        S: AsRef<str>,
        A: AsRef<str>,
        R: Borrow<FluentResource>,
        M: MemoizerKind,
    {
        let mut scope = Scope::new(self, args);
        pattern.write(w, &mut scope)
    }
}
