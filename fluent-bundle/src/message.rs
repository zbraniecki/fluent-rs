use fluent_syntax::ast;

#[derive(Debug, PartialEq)]
pub struct FluentAttribute<'m, S> {
    pub id: &'m str,
    pub value: &'m ast::Pattern<&'m S>,
}
/// A single localization unit composed of an identifier,
/// value, and attributes.
#[derive(Debug, PartialEq)]
pub struct FluentMessage<'m, S> {
    pub value: Option<&'m ast::Pattern<S>>,
    pub attributes: Vec<FluentAttribute<'m, S>>,
}

impl<'m, S> FluentMessage<'m, S> {
    pub fn get_attribute(&self, key: &str) -> Option<&FluentAttribute<S>> {
        self.attributes.iter().find(|attr| attr.id == key)
    }
}
