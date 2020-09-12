use super::scope::Scope;
use super::WriteValue;

use std::borrow::Borrow;
use std::fmt;

use fluent_syntax::ast;

use crate::memoizer::MemoizerKind;
use crate::resolver::{ResolveValue, ResolverError};
use crate::resource::FluentResource;
use crate::types::FluentValue;

impl<'p> WriteValue for ast::Expression<'p> {
    fn write<'scope, W, R, M: MemoizerKind>(
        &'scope self,
        w: &mut W,
        scope: &mut Scope<'scope, R, M>,
    ) -> fmt::Result
    where
        W: fmt::Write,
        R: Borrow<FluentResource>,
    {
        match self {
            ast::Expression::InlineExpression(exp) => exp.write(w, scope),
            ast::Expression::SelectExpression { selector, variants } => {
                let selector = selector.resolve(scope);
                match selector {
                    FluentValue::String(_) | FluentValue::Number(_) => {
                        for variant in variants {
                            let key = match variant.key {
                                ast::VariantKey::Identifier { name } => name.into(),
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
                scope.errors.push(ResolverError::MissingDefault);
                Ok(())
            }
        }
    }
}
