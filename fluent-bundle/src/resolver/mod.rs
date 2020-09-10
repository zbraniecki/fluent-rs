pub(crate) mod scope;

use crate::memoizer::MemoizerKind;
use crate::resource::FluentResource;
use crate::types::FluentValue;
use fluent_syntax::ast;
use scope::Scope;
use std::borrow::Borrow;
use std::fmt;

pub(crate) trait ResolveValue<'source, S> {
    fn write<W, R, M, A>(&'source self, w: &mut W, scope: &Scope<'source, R, M, A>) -> fmt::Result
    where
        W: fmt::Write,
        M: MemoizerKind,
        R: Borrow<FluentResource>,
        A: AsRef<str>;

    fn resolve<'s, R, M, A>(&self, scope: &'s Scope<R, M, A>) -> FluentValue<&'s str>
    where
        A: AsRef<str>,
    {
        panic!()
        // unimplemented!()
    }
}

impl<'source, S> ResolveValue<'source, S> for ast::Pattern<S>
where
    S: AsRef<str>,
{
    fn write<W, R, M, A>(&'source self, w: &mut W, scope: &Scope<'source, R, M, A>) -> fmt::Result
    where
        W: fmt::Write,
        M: MemoizerKind,
        R: Borrow<FluentResource>,
        A: AsRef<str>,
    {
        for elem in &self.elements {
            match elem {
                ast::PatternElement::TextElement(t) => {
                    w.write_str(t.as_ref())?;
                }
                ast::PatternElement::Placeable(expr) => {
                    expr.write(w, scope)?;
                }
            }
        }
        Ok(())
    }
}

impl<'source, S> ResolveValue<'source, S> for ast::Expression<S>
where
    S: AsRef<str>,
{
    fn write<W, R, M, A>(&'source self, w: &mut W, scope: &Scope<'source, R, M, A>) -> fmt::Result
    where
        W: fmt::Write,
        M: MemoizerKind,
        R: Borrow<FluentResource>,
        A: AsRef<str>,
    {
        match self {
            ast::Expression::InlineExpression(expr) => expr.write(w, scope),
            ast::Expression::SelectExpression { selector, variants } => {
                let selector = selector.resolve(scope);
                match selector {
                    FluentValue::String(_) | FluentValue::Number(_) => {
                        for variant in variants {
                            let key = match &variant.key {
                                ast::VariantKey::Identifier { name } => {
                                    FluentValue::String(name.as_ref())
                                }
                                ast::VariantKey::NumberLiteral { value } => {
                                    FluentValue::try_number(value.as_ref())
                                }
                            };
                            if key.matches(&selector, scope) {
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
                // scope.errors.push(ResolverError::MissingDefault);
                // FluentValue::None
                panic!()
            }
        }
    }
}

impl<'source, S> ResolveValue<'source, S> for ast::InlineExpression<S>
where
    S: AsRef<str>,
{
    fn write<W, R, M, A>(&'source self, w: &mut W, scope: &Scope<'source, R, M, A>) -> fmt::Result
    where
        W: fmt::Write,
        M: MemoizerKind,
        R: Borrow<FluentResource>,
        A: AsRef<str>,
    {
        match self {
            // ast::InlineExpression::StringLiteral { value } => unescape_unicode(value).into(),
            // ast::InlineExpression::MessageReference { id, attribute } => scope
            //     .bundle
            //     .get_entry_message(&id.name)
            //     .and_then(|msg| {
            //         if let Some(attr) = attribute {
            //             msg.attributes
            //                 .iter()
            //                 .find(|a| a.id.name == attr.name)
            //                 .map(|attr| scope.track(&attr.value, self.into()))
            //         } else {
            //             msg.value
            //                 .as_ref()
            //                 .map(|value| scope.track(value, self.into()))
            //         }
            //     })
            //     .unwrap_or_else(|| generate_ref_error(scope, self.into())),
            // ast::InlineExpression::NumberLiteral { value } => FluentValue::try_number(*value),
            // ast::InlineExpression::TermReference {
            //     id,
            //     attribute,
            //     arguments,
            // } => {
            //     let (_, resolved_named_args) = get_arguments(scope, arguments);

            //     scope.local_args = Some(resolved_named_args);

            //     let value = scope
            //         .bundle
            //         .get_entry_term(&id.name)
            //         .and_then(|term| {
            //             if let Some(attr) = attribute {
            //                 term.attributes
            //                     .iter()
            //                     .find(|a| a.id.name == attr.name)
            //                     .map(|attr| scope.track(&attr.value, self.into()))
            //             } else {
            //                 Some(scope.track(&term.value, self.into()))
            //             }
            //         })
            //         .unwrap_or_else(|| generate_ref_error(scope, self.into()));

            //     scope.local_args = None;
            //     value
            // }
            // ast::InlineExpression::FunctionReference { id, arguments } => {
            //     let (resolved_positional_args, resolved_named_args) =
            //         get_arguments(scope, arguments);

            //     let func = scope.bundle.get_entry_function(id.name);

            //     if let Some(func) = func {
            //         func(resolved_positional_args.as_slice(), &resolved_named_args)
            //     } else {
            //         generate_ref_error(scope, self.into())
            //     }
            // }
            ast::InlineExpression::VariableReference { id } => {
                if let Some(args) = &scope.args {
                    let arg = args.get(id.name.as_ref());
                    if let Some(arg) = arg {
                        arg.write(w)
                    } else {
                        w.write_str("???")
                    }
                } else {
                    Ok(())
                }
            }
            _ => unimplemented!(),
            // ast::InlineExpression::Placeable { expression } => expression.resolve(scope),
        }
    }

    fn resolve<'s, R, M, A>(&self, scope: &'s Scope<R, M, A>) -> FluentValue<&'s str>
    where
        A: AsRef<str>,
    {
        match self {
            ast::InlineExpression::VariableReference { id } => {
                if let Some(args) = &scope.args {
                    let arg = args.get(id.name.as_ref());
                    if let Some(arg) = arg {
                        arg.borrowed()
                    } else {
                        panic!()
                    }
                } else {
                    panic!()
                }
            }
            _ => unimplemented!(),
        }
    }
}
