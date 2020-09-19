use super::scope::Scope;
use super::{ResolverError, WriteValue};

use std::borrow::Borrow;
use std::fmt;

use fluent_syntax::{ast, parser::Slice};

use crate::memoizer::MemoizerKind;
use crate::resolver::ResolveValue;
use crate::resource::FluentResource;
use crate::types::FluentValue;

const MAX_PLACEABLES: u8 = 100;

impl<S> WriteValue<S> for ast::Pattern<S> {
    fn write<'scope, 'errors, W, R, M: MemoizerKind>(
        &'scope self,
        w: &mut W,
        scope: &mut Scope<'scope, 'errors, R, M, S>,
    ) -> fmt::Result
    where
        W: fmt::Write,
        R: Borrow<FluentResource>,
        S: Slice<'scope>,
    {
        let len = self.elements.len();

        for elem in &self.elements {
            if scope.dirty {
                return Ok(());
            }

            match elem {
                ast::PatternElement::TextElement { value } => {
                    if let Some(ref transform) = scope.bundle.transform {
                        w.write_str(&transform(value.as_str()))?;
                    } else {
                        w.write_str(value.as_str())?;
                    }
                }
                ast::PatternElement::Placeable { ref expression } => {
                    scope.placeables += 1;
                    if scope.placeables > MAX_PLACEABLES {
                        scope.dirty = true;
                        scope.add_error(ResolverError::TooManyPlaceables);
                        return Ok(());
                    }

                    let needs_isolation = scope.bundle.use_isolating
                        && len > 1
                        && match expression {
                            ast::Expression::InlineExpression(
                                ast::InlineExpression::MessageReference { .. },
                            )
                            | ast::Expression::InlineExpression(
                                ast::InlineExpression::TermReference { .. },
                            )
                            | ast::Expression::InlineExpression(
                                ast::InlineExpression::StringLiteral { .. },
                            ) => false,
                            _ => true,
                        };
                    if needs_isolation {
                        w.write_char('\u{2068}')?;
                    }
                    scope.maybe_track(w, self, expression)?;
                    if needs_isolation {
                        w.write_char('\u{2069}')?;
                    }
                }
            }
        }
        Ok(())
    }

    fn write_error<'scope, W>(&self, _w: &mut W) -> fmt::Result
    where
        W: fmt::Write,
        S: Slice<'scope>,
    {
        unreachable!()
    }
}

impl<S> ResolveValue<S> for ast::Pattern<S> {
    fn resolve<'source, 'errors, R, M: MemoizerKind>(
        &'source self,
        scope: &mut Scope<'source, 'errors, R, M, S>,
    ) -> FluentValue<'source>
    where
        R: Borrow<FluentResource>,
        S: Slice<'source>,
    {
        let len = self.elements.len();

        if len == 1 {
            if let ast::PatternElement::TextElement { ref value } = self.elements[0] {
                return scope.bundle.transform.map_or_else(
                    || value.as_str().into(),
                    |transform| transform(value.as_str()).into(),
                );
            }
        }

        let mut result = String::new();
        self.write(&mut result, scope)
            .expect("Failed to write to a string.");
        result.into()
    }

    fn resolve_error<'scope>(&self) -> String
    where
        S: Slice<'scope>,
    {
        unreachable!()
    }
}
