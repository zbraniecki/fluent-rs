use fluent_syntax::ast;
use fluent_syntax::parser::parse;
use fluent_syntax::parser::ParserError;

pub struct FluentResource<'resource> {
    pub ast: ast::Resource<'resource>,
}

impl<'resource> FluentResource<'resource> {
    pub fn from_string(source: &'resource str) -> Result<Self, (Self, Vec<ParserError>)> {
        match parse(&source) {
            Ok(ast) => Ok(FluentResource { ast }),
            Err((ast, errors)) => Err((FluentResource { ast }, errors)),
        }
    }
}
