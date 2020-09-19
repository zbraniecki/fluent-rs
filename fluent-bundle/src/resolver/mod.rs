//! The `ResolveValue` trait resolves Fluent AST nodes to [`FluentValues`].
//!
//! This is an internal API used by [`FluentBundle`] to evaluate Messages, Attributes and other
//! AST nodes to [`FluentValues`] which can be then formatted to strings.
//!
//! [`FluentValues`]: ../types/enum.FluentValue.html
//! [`FluentBundle`]: ../bundle/struct.FluentBundle.html

mod errors;
mod expression;
mod inline_expression;
mod pattern;
mod scope;

pub use errors::ResolverError;
pub use scope::Scope;

use std::borrow::Borrow;
use std::fmt;

use fluent_syntax::parser::Slice;

use crate::memoizer::MemoizerKind;
use crate::resource::FluentResource;
use crate::types::FluentValue;

// Converts an AST node to a `FluentValue`.
pub(crate) trait ResolveValue<S> {
    fn resolve<'source, 'errors, R, M: MemoizerKind>(
        &'source self,
        scope: &mut Scope<'source, 'errors, R, M, S>,
    ) -> FluentValue<'source>
    where
        R: Borrow<FluentResource>,
        S: Slice<'source>;

    fn resolve_error<'source>(&self) -> String
    where
        S: Slice<'source>;
}

pub(crate) trait WriteValue<S> {
    fn write<'source, 'errors, W, R, M: MemoizerKind>(
        &'source self,
        w: &mut W,
        scope: &mut Scope<'source, 'errors, R, M, S>,
    ) -> fmt::Result
    where
        W: fmt::Write,
        R: Borrow<FluentResource>,
        S: Slice<'source>;

    fn write_error<'source, W>(&self, w: &mut W) -> fmt::Result
    where
        W: fmt::Write,
        S: Slice<'source>;
}
