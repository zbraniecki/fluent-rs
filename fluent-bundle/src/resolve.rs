use fluent_syntax::ast;
use fluent_syntax::parser::Slice;
use std::fmt;

pub(crate) trait ResolveValue<'s> {
    fn write<W: fmt::Write>(&self, w: &mut W) -> fmt::Result;
}

impl<'s, S> ResolveValue<'s> for ast::Pattern<S>
where
    S: Slice<'s>,
{
    fn write<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        for elem in &self.elements {
            match elem {
                ast::PatternElement::TextElement(t) => {
                    t.write(w)?;
                }
                _ => unimplemented!(),
            }
        }
        Ok(())
    }
}
