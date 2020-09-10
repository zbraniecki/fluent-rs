use crate::memoizer::{Memoizer, MemoizerKind};
use crate::resolve::ResolveValue;
use crate::resource::FluentResource;
use crate::FluentError;
use fluent_syntax::ast;
use std::borrow::Borrow;
use std::fmt;

pub struct FluentBundle<R, M> {
    resources: Vec<R>,
    intls: M,
}

impl<R> FluentBundle<R, Memoizer> {
    pub fn new() -> Self {
        FluentBundle {
            resources: vec![],
            intls: Memoizer::new(),
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

    pub fn format_pattern<'bundle, S, W: fmt::Write>(
        &'bundle self,
        w: &mut W,
        pattern: &ast::Pattern<S>,
    ) -> fmt::Result
    where
        S: AsRef<str>,
    {
        pattern.write(w)
    }
}
