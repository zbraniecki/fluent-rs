use fluent_syntax::ast;
use std::fmt;

pub(crate) trait ResolveValue<'s> {
    fn write<W: fmt::Write>(&self, w: &mut W) -> fmt::Result;
}

impl<'s, S> ResolveValue<'s> for ast::Pattern<S>
where
    S: AsRef<str>,
{
    fn write<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        for elem in &self.elements {
            match elem {
                ast::PatternElement::TextElement(t) => {
                    w.write_str(t.as_ref())?;
                }
                _ => unimplemented!(),
            }
        }
        Ok(())
    }
}
