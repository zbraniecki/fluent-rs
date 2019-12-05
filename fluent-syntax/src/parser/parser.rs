use super::ast;
use super::lexer::Lexer;
use super::lexer::Token;
use fallible_iterator::FallibleIterator;
use fallible_iterator::Peekable;
use std::io::Bytes;
use std::ops::Range;

pub struct Parser<R>
where
    R: std::io::Read,
{
    lexer: Peekable<Lexer<R>>,
}

impl<R> Parser<R>
where
    R: std::io::Read,
{
    pub fn new(source: Bytes<R>) -> Self {
        Parser {
            lexer: Lexer::new(source).peekable(),
        }
    }
}

impl<R> Iterator for Parser<R>
where
    R: std::io::Read,
{
    type Item = Result<ast::Entry, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
