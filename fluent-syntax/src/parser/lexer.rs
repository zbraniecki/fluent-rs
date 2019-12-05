use fallible_iterator::FallibleIterator;
use std::io::Bytes;
use std::iter::Enumerate;
use std::iter::Peekable;
use std::ops::Range;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    EqSign,
    CommentSign,
    // GroupCommentSign,
    // ResourceCommentSign,
    Eol,
    Eot, // End Of Text
    // Dot,
    // MinusSign,
    Text(usize, String),
}

#[derive(Debug, PartialEq)]
pub enum LexerState {
    Resource,
    Message, // Or Term
    Pattern,
    Comment,
}

#[derive(Debug, PartialEq)]
pub enum NextLine {
    TextContinuation,
    Attribute,
    NewEntry,
}

pub enum Buffer {
    Token(Token),
    Byte(u8),
}

type LexerResult<R> = Result<R, std::io::Error>;
type LexerMaybeResult<R> = Result<Option<R>, std::io::Error>;

pub struct Lexer<R>
where
    R: std::io::Read,
{
    pub state: LexerState,
    pub source: Bytes<R>,
    pub buffer: Option<Buffer>,
}

impl<R> Lexer<R>
where
    R: std::io::Read,
{
    pub fn new(source: Bytes<R>) -> Self {
        Lexer {
            state: LexerState::Resource,
            source,
            buffer: None,
        }
    }

    fn get_ident(&mut self, b: u8) -> LexerResult<Token> {
        let mut buffer = vec![b];
        while let Some(b) = self.source.next() {
            let b = b?;
            if !b.is_ascii_alphanumeric() {
                self.buffer = Some(Buffer::Byte(b));
                break;
            }
            buffer.push(b);
        }
        Ok(Token::Identifier(String::from_utf8(buffer).unwrap()))
    }

    fn tokenize_resource(&mut self, b: u8) -> LexerMaybeResult<Token> {
        match b {
            b'#' => {
                self.state = LexerState::Comment;
                Ok(Some(Token::CommentSign))
            }
            b if b.is_ascii_alphabetic() => {
                self.state = LexerState::Message;
                Ok(Some(self.get_ident(b)?))
            }
            b'\n' => Ok(Some(Token::Eol)),
            _ => panic!(),
        }
    }

    fn tokenize_message(&mut self, b: u8) -> LexerMaybeResult<Token> {
        let mut b = b;
        while b == b' ' {
            b = match self.maybe_take()? {
                Some(b) => b,
                None => return Ok(None),
            };
        }

        match b {
            b'=' => {
                self.state = LexerState::Pattern;
                return Ok(Some(Token::EqSign));
            }
            _ => panic!(),
        }
    }

    fn tokenize_pattern(&mut self, b: u8) -> LexerResult<Token> {
        let mut indent = 0;
        let mut in_indent = true;

        let mut b = Some(b);
        while b == Some(b' ') {
            indent += 1;
            b = self.maybe_take()?;
        }

        let mut buffer = vec![];

        loop {
            match b {
                Some(b'\n') => {
                    break;
                }
                Some(cc) => {
                    buffer.push(cc);
                    b = self.maybe_take()?;
                }
                None => {
                    break;
                }
            }
        }
        self.buffer = Some(Buffer::Token(Token::Eot));
        self.state = LexerState::Resource;
        return Ok(Token::Text(indent, String::from_utf8(buffer).unwrap()));
    }

    fn tokenize_comment(&mut self, b: u8) -> LexerResult<Token> {
        if b != b' ' {
            if b != b'\n' {
                panic!();
            }
            self.state = LexerState::Resource;
            return Ok(Token::Text(0, String::new()));
        }

        let mut buffer = vec![b];

        loop {
            match self.maybe_take()? {
                Some(b'\n') => {
                    break;
                }
                Some(b) => {
                    buffer.push(b);
                }
                None => {
                    break;
                }
            }
        }
        self.state = LexerState::Resource;
        Ok(Token::Text(0, String::from_utf8(buffer).unwrap()))
    }

    fn consume(&mut self, b: u8) -> LexerMaybeResult<Token> {
        match self.state {
            LexerState::Resource => self.tokenize_resource(b),
            LexerState::Message => self.tokenize_message(b),
            LexerState::Pattern => Ok(Some(self.tokenize_pattern(b)?)),
            LexerState::Comment => Ok(Some(self.tokenize_comment(b)?)),
            _ => panic!(),
        }
    }

    fn maybe_take(&mut self) -> LexerMaybeResult<u8> {
        match self.source.next() {
            Some(Ok(b)) => Ok(Some(b)),
            Some(Err(err)) => Err(err),
            None => Ok(None),
        }
    }
}

impl<R> FallibleIterator for Lexer<R>
where
    R: std::io::Read,
{
    type Item = Token;
    type Error = std::io::Error;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let x = match self.buffer.take() {
            Some(Buffer::Token(token)) => {
                return Ok(Some(token));
            }
            Some(Buffer::Byte(b)) => self.consume(b),
            None => {
                let byte = self.source.next();
                match byte {
                    Some(Ok(b)) => self.consume(b),
                    Some(Err(err)) => Err(err),
                    None => Ok(None),
                }
            }
        };
        dbg!(x)
    }
}
