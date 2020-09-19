use super::scope::Scope;
use super::{ResolveValue, ResolverError, WriteValue};

use std::borrow::Borrow;
use std::fmt;

use fluent_syntax::unicode::{unescape_unicode, unescape_unicode_to_string};
use fluent_syntax::{ast, parser::Slice};

use crate::entry::GetEntry;
use crate::memoizer::MemoizerKind;
use crate::resource::FluentResource;
use crate::types::FluentValue;

impl<S> WriteValue<S> for ast::InlineExpression<S> {
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
            ast::InlineExpression::StringLiteral { value } => unescape_unicode(w, value.as_str()),
            ast::InlineExpression::MessageReference { id, attribute } => scope
                .bundle
                .get_entry_message(id.name.as_str())
                .and_then(|msg| {
                    if let Some(attr) = attribute {
                        // msg.attributes
                        //     .iter()
                        //     .find(|a| a.id.name == attr.name.as_str())
                        //     .map(|attr| scope.track(w, attr.value, self))
                        panic!();
                    } else {
                        panic!();
                        //msg.value.as_ref().map(|value| scope.track(w, value, self))
                    }
                })
                .unwrap_or_else(|| scope.write_ref_error(w, self)),
            ast::InlineExpression::NumberLiteral { value } => {
                FluentValue::try_number(value).write(w, scope)
            }
            ast::InlineExpression::TermReference {
                id,
                attribute,
                arguments,
            } => {
                let (_, resolved_named_args) = scope.get_arguments(arguments);

                panic!();
                // scope.local_args = Some(resolved_named_args);
                // let result = scope
                //     .bundle
                //     .get_entry_term(&id.name)
                //     .and_then(|term| {
                //         if let Some(attr) = attribute {
                //             term.attributes
                //                 .iter()
                //                 .find(|a| a.id.name == attr.name)
                //                 .map(|attr| scope.track(w, &attr.value, self))
                //         } else {
                //             Some(scope.track(w, &term.value, self))
                //         }
                //     })
                //     .unwrap_or_else(|| scope.write_ref_error(w, self));
                // scope.local_args = None;
                // result
            }
            ast::InlineExpression::FunctionReference { id, arguments } => {
                let (resolved_positional_args, resolved_named_args) =
                    scope.get_arguments(arguments);

                let func = scope.bundle.get_entry_function(id.name.as_str());

                if let Some(func) = func {
                    let result = func(resolved_positional_args.as_slice(), &resolved_named_args);
                    if let FluentValue::Error = result {
                        self.write_error(w)
                    } else {
                        w.write_str(&result.as_string(scope))
                    }
                } else {
                    scope.write_ref_error(w, self)
                }
            }
            ast::InlineExpression::VariableReference { id } => {
                let args = scope.local_args.as_ref().or(scope.args);

                if let Some(arg) = args.and_then(|args| args.get(id.name.as_str())) {
                    arg.write(w, scope)
                } else {
                    if scope.local_args.is_none() {
                        scope.add_error(ResolverError::Reference(self.resolve_error()));
                    }
                    w.write_char('{')?;
                    self.write_error(w)?;
                    w.write_char('}')
                }
            }
            ast::InlineExpression::Placeable { expression } => expression.write(w, scope),
        }
    }

    fn write_error<'scope, W>(&self, w: &mut W) -> fmt::Result
    where
        W: fmt::Write,
        S: Slice<'scope>,
    {
        match self {
            ast::InlineExpression::MessageReference {
                id,
                attribute: Some(attribute),
            } => write!(w, "{}.{}", id.name, attribute.name),
            ast::InlineExpression::MessageReference {
                id,
                attribute: None,
            } => w.write_str(id.name.as_str()),
            ast::InlineExpression::TermReference {
                id,
                attribute: Some(attribute),
                ..
            } => write!(w, "-{}.{}", id.name, attribute.name),
            ast::InlineExpression::TermReference {
                id,
                attribute: None,
                ..
            } => write!(w, "-{}", id.name),
            ast::InlineExpression::FunctionReference { id, .. } => write!(w, "{}()", id.name),
            ast::InlineExpression::VariableReference { id } => write!(w, "${}", id.name),
            _ => unreachable!(),
        }
    }
}

impl<S> ResolveValue<S> for ast::InlineExpression<S> {
    fn resolve<'source, 'errors, R, M: MemoizerKind>(
        &'source self,
        scope: &mut Scope<'source, 'errors, R, M, S>,
    ) -> FluentValue<'source>
    where
        R: Borrow<FluentResource>,
        S: Slice<'source>,
    {
        match self {
            ast::InlineExpression::StringLiteral { value } => {
                unescape_unicode_to_string(value.as_str()).into()
            }
            ast::InlineExpression::NumberLiteral { value } => FluentValue::try_number(value),
            ast::InlineExpression::VariableReference { id } => {
                let args = scope.local_args.as_ref().or(scope.args);

                if let Some(arg) = args.and_then(|args| args.get(id.name.as_str())) {
                    arg.clone()
                } else {
                    if scope.local_args.is_none() {
                        scope.add_error(ResolverError::Reference(self.resolve_error()));
                    }
                    FluentValue::Error
                }
            }
            _ => {
                let mut result = String::new();
                self.write(&mut result, scope).expect("Failed to write");
                result.into()
            }
        }
    }

    fn resolve_error<'source>(&self) -> String
    where
        S: Slice<'source>,
    {
        match self {
            ast::InlineExpression::MessageReference {
                attribute: None, ..
            } => {
                let mut error = String::from("Unknown message: ");
                self.write_error(&mut error)
                    .expect("Failed to write to String.");
                error
            }
            ast::InlineExpression::MessageReference { .. } => {
                let mut error = String::from("Unknown attribute: ");
                self.write_error(&mut error)
                    .expect("Failed to write to String.");
                error
            }
            ast::InlineExpression::VariableReference { .. } => {
                let mut error = String::from("Unknown variable: ");
                self.write_error(&mut error)
                    .expect("Failed to write to String.");
                error
            }
            ast::InlineExpression::TermReference {
                attribute: None, ..
            } => {
                let mut error = String::from("Unknown term: ");
                self.write_error(&mut error)
                    .expect("Failed to write to String.");
                error
            }
            ast::InlineExpression::TermReference { .. } => {
                let mut error = String::from("Unknown attribute: ");
                self.write_error(&mut error)
                    .expect("Failed to write to String.");
                error
            }
            ast::InlineExpression::FunctionReference { .. } => {
                let mut error = String::from("Unknown function: ");
                self.write_error(&mut error)
                    .expect("Failed to write to String.");
                error
            }
            _ => unreachable!(),
        }
    }
}
