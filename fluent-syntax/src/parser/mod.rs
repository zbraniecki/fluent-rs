use crate::ast;
use std::fmt;

mod errors;
pub use errors::ParserError;

pub trait Slice<'s>: PartialEq<&'s str> {
    fn slice(&self, start: usize, end: usize) -> Self;
    fn write<W: fmt::Write>(&self, w: &mut W) -> fmt::Result;
}

impl<'s> Slice<'s> for String {
    fn slice(&self, start: usize, end: usize) -> String {
        self[start..end].to_string()
    }

    fn write<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        w.write_str(&self)
    }
}

impl<'s> Slice<'s> for &'s str {
    fn slice(&self, start: usize, end: usize) -> &'s str {
        &self[start..end]
    }

    fn write<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        w.write_str(self)
    }
}

pub struct Parser<R> {
    source: R,
    ptr: usize,
}

impl<'s, R> Parser<R>
    where R: Slice<'s> {
    pub fn new(source: R) -> Self {
        Self {
            source,
            ptr: 0,
        }
    }

    pub fn parse(&mut self) -> Result<ast::Resource<R>, (ast::Resource<R>, Vec<ParserError>)> {
        let msg = self.get_message();
        let entry = ast::Entry::Message(msg);
        let res = ast::Resource {
            body: vec![
                ast::ResourceEntry::Entry(entry)
            ]
        };
        Ok(res)
    }

    fn get_message(&mut self) -> ast::Message<R> {
        let id = self.get_identifier();
        self.ptr += 1;
        let value = self.get_value();
        ast::Message {
            id,
            value,
            attributes: vec![],
            comment: None
        }
    }

    fn get_identifier(&mut self) -> ast::Identifier<R> {
        ast::Identifier { name: self.source.slice(0, 3) }
    }

    fn get_value(&mut self) -> Option<ast::Pattern<R>> {
        Some(ast::Pattern {
            elements: vec![
                ast::PatternElement::TextElement(self.source.slice(4, 9))
            ]
        })
    }
}
