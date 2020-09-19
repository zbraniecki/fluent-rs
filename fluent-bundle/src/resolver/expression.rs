use super::scope::Scope;
use super::WriteValue;

use std::borrow::Borrow;
use std::fmt;

use fluent_syntax::{ast, parser::Slice};

use crate::memoizer::MemoizerKind;
use crate::resolver::{ResolveValue, ResolverError};
use crate::resource::FluentResource;
use crate::types::FluentValue;

impl<S> WriteValue<S> for ast::Expression<S> {
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
        match self {
            ast::Expression::InlineExpression(exp) => exp.write(w, scope),
            ast::Expression::SelectExpression { selector, variants } => {
                let selector = selector.resolve(scope);
                match selector {
                    FluentValue::String(_) | FluentValue::Number(_) => {
                        for variant in variants {
                            let key = match &variant.key {
                                // ast::VariantKey::Identifier { name } => name.into(),
                                ast::VariantKey::Identifier { name } => panic!(),
                                ast::VariantKey::NumberLiteral { value } => {
                                    FluentValue::try_number(value)
                                }
                            };
                            if key.matches(&selector, &scope) {
                                return variant.value.write(w, scope);
                            }
                        }
                    }
                    _ => {}
                }

                for variant in variants {
                    if variant.default {
                        return variant.value.write(w, scope);
                    }
                }
                scope.add_error(ResolverError::MissingDefault);
                Ok(())
            }
        }
    }

    fn write_error<'scope, W>(&self, w: &mut W) -> fmt::Result
    where
        W: fmt::Write,
        S: Slice<'scope>,
    {
        match self {
            ast::Expression::InlineExpression(exp) => exp.write_error(w),
            ast::Expression::SelectExpression { .. } => unreachable!(),
        }
    }
}
