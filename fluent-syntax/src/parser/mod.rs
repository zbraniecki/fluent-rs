use crate::ast;
use std::fmt;
use std::ops::Range;
use std::result;

#[macro_use]
mod errors;
pub use errors::{ErrorKind, ParserError};

pub trait Slice<'s>: PartialEq<&'s str> {
    fn slice(&self, range: Range<usize>) -> Self;
    fn write<W: fmt::Write>(&self, w: &mut W) -> fmt::Result;
    fn get_char(&self, idx: usize) -> Option<&u8>;
    fn length(&self) -> usize;
    fn trim(&mut self);
}

impl<'s> Slice<'s> for String {
    fn slice(&self, range: Range<usize>) -> String {
        self[range].to_string()
    }

    fn write<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        w.write_str(&self)
    }

    fn get_char(&self, idx: usize) -> Option<&u8> {
        self.as_bytes().get(idx)
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn trim(&mut self) {
        *self = self.trim_end().to_string();
    }
}

impl<'s> Slice<'s> for &'s str {
    fn slice(&self, range: Range<usize>) -> &'s str {
        &self[range]
    }

    fn write<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        w.write_str(self)
    }

    fn get_char(&self, idx: usize) -> Option<&u8> {
        self.as_bytes().get(idx)
    }

    fn length(&self) -> usize {
        self.len()
    }

    fn trim(&mut self) {
        *self = self.trim_end();
    }
}

pub type Result<T> = result::Result<T, ParserError>;

#[derive(Debug, PartialEq)]
enum TextElementTermination {
    LineFeed,
    CRLF,
    PlaceableStart,
    EOF,
}

// This enum tracks the placement of the text element in the pattern, which is needed for
// dedentation logic.
#[derive(Debug, PartialEq)]
enum TextElementPosition {
    InitialLineStart,
    LineStart,
    Continuation,
}

// This enum allows us to mark pointers in the source which will later become text elements
// but without slicing them out of the source string. This makes the indentation adjustments
// cheaper since they'll happen on the pointers, rather than extracted slices.
#[derive(Debug)]
enum PatternElementPlaceholders<S> {
    Placeable(ast::Expression<S>),
    // (start, end, indent, position)
    TextElement(usize, usize, usize, TextElementPosition),
}

// This enum tracks whether the text element is blank or not.
// This is important to identify text elements which should not be taken into account
// when calculating common indent.
#[derive(Debug, PartialEq)]
enum TextElementType {
    Blank,
    NonBlank,
}

pub struct Parser<S> {
    source: S,
    ptr: usize,
    length: usize,
}

impl<'s, S> Parser<S>
where
    S: Slice<'s>,
{
    pub fn new(source: S) -> Self {
        let length = source.length();
        Self {
            source,
            ptr: 0,
            length,
        }
    }

    pub fn parse(
        &mut self,
    ) -> std::result::Result<ast::Resource<S>, (ast::Resource<S>, Vec<ParserError>)> {
        let msg = self.get_message().unwrap();
        let entry = ast::Entry::Message(msg);
        let res = ast::Resource {
            body: vec![ast::ResourceEntry::Entry(entry)],
        };
        Ok(res)
    }

    fn get_message(&mut self) -> Result<ast::Message<S>> {
        let id = self.get_identifier()?;
        self.skip_blank_inline();
        self.expect_byte(b'=')?;
        let value = self.get_pattern()?;

        self.skip_blank_block();
        Ok(ast::Message {
            id,
            value,
            attributes: vec![],
            comment: None,
        })
    }

    fn get_identifier(&mut self) -> Result<ast::Identifier<S>> {
        let mut ptr = self.ptr;

        match self.source.get_char(ptr) {
            Some(b) if b.is_ascii_alphabetic() => {
                ptr += 1;
            }
            _ => {
                return error!(
                    ErrorKind::ExpectedCharRange {
                        range: "a-zA-Z".to_string()
                    },
                    ptr
                );
            }
        }

        while let Some(b) = self.source.get_char(ptr) {
            if b.is_ascii_alphabetic() || b.is_ascii_digit() || [b'_', b'-'].contains(b) {
                ptr += 1;
            } else {
                break;
            }
        }

        let name = self.source.slice(self.ptr..ptr);
        self.ptr = ptr;

        Ok(ast::Identifier { name })
    }

    fn get_pattern(&mut self) -> Result<Option<ast::Pattern<S>>> {
        let mut elements = vec![];
        let mut last_non_blank = None;
        let mut common_indent = None;

        self.skip_blank_inline();

        let mut text_element_role = if self.skip_eol() {
            self.skip_blank_block();
            TextElementPosition::LineStart
        } else {
            TextElementPosition::InitialLineStart
        };

        while self.ptr < self.length {
            if self.is_current_byte(b'{') {
                if text_element_role == TextElementPosition::LineStart {
                    common_indent = Some(0);
                }
                let exp = self.get_placeable()?;
                last_non_blank = Some(elements.len());
                elements.push(PatternElementPlaceholders::Placeable(exp));
                text_element_role = TextElementPosition::Continuation;
            } else {
                let slice_start = self.ptr;
                let mut indent = 0;
                if text_element_role == TextElementPosition::LineStart {
                    indent = self.skip_blank_inline();
                    if self.ptr >= self.length {
                        break;
                    }
                    let b = self.source.get_char(self.ptr);
                    if indent == 0 {
                        if b != Some(&b'\n') {
                            break;
                        }
                    } else if !self.is_byte_pattern_continuation(*b.unwrap()) {
                        self.ptr = slice_start;
                        break;
                    }
                }
                let (start, end, text_element_type, termination_reason) = self.get_text_slice()?;
                if start != end {
                    if text_element_role == TextElementPosition::LineStart
                        && text_element_type == TextElementType::NonBlank
                    {
                        if let Some(common) = common_indent {
                            if indent < common {
                                common_indent = Some(indent);
                            }
                        } else {
                            common_indent = Some(indent);
                        }
                    }
                    if text_element_role != TextElementPosition::LineStart
                        || text_element_type == TextElementType::NonBlank
                        || termination_reason == TextElementTermination::LineFeed
                    {
                        if text_element_type == TextElementType::NonBlank {
                            last_non_blank = Some(elements.len());
                        }
                        elements.push(PatternElementPlaceholders::TextElement(
                            slice_start,
                            end,
                            indent,
                            text_element_role,
                        ));
                    }
                }

                text_element_role = match termination_reason {
                    TextElementTermination::LineFeed => TextElementPosition::LineStart,
                    TextElementTermination::CRLF => TextElementPosition::Continuation,
                    TextElementTermination::PlaceableStart => TextElementPosition::Continuation,
                    TextElementTermination::EOF => TextElementPosition::Continuation,
                };
            }
        }

        if let Some(last_non_blank) = last_non_blank {
            let elements = elements
                .into_iter()
                .take(last_non_blank + 1)
                .enumerate()
                .map(|(i, elem)| match elem {
                    PatternElementPlaceholders::Placeable(exp) => {
                        ast::PatternElement::Placeable(exp)
                    }
                    PatternElementPlaceholders::TextElement(start, end, indent, role) => {
                        let start = if role == TextElementPosition::LineStart {
                            if let Some(common_indent) = common_indent {
                                start + std::cmp::min(indent, common_indent)
                            } else {
                                start + indent
                            }
                        } else {
                            start
                        };
                        let mut slice = self.source.slice(start..end);
                        if last_non_blank == i {
                            slice.trim();
                            ast::PatternElement::TextElement(slice)
                        } else {
                            ast::PatternElement::TextElement(slice)
                        }
                    }
                })
                .collect();
            return Ok(Some(ast::Pattern { elements }));
        }

        Ok(None)
    }

    fn get_text_slice(
        &mut self,
    ) -> Result<(usize, usize, TextElementType, TextElementTermination)> {
        let start_pos = self.ptr;
        let mut text_element_type = TextElementType::Blank;

        while let Some(b) = self.source.get_char(self.ptr) {
            match b {
                b' ' => self.ptr += 1,
                b'\n' => {
                    self.ptr += 1;
                    return Ok((
                        start_pos,
                        self.ptr,
                        text_element_type,
                        TextElementTermination::LineFeed,
                    ));
                }
                b'\r' if self.is_byte_at(b'\n', self.ptr + 1) => {
                    self.ptr += 1;
                    return Ok((
                        start_pos,
                        self.ptr - 1,
                        text_element_type,
                        TextElementTermination::CRLF,
                    ));
                }
                b'{' => {
                    return Ok((
                        start_pos,
                        self.ptr,
                        text_element_type,
                        TextElementTermination::PlaceableStart,
                    ));
                }
                b'}' => {
                    return error!(ErrorKind::UnbalancedClosingBrace, self.ptr);
                }
                _ => {
                    text_element_type = TextElementType::NonBlank;
                    self.ptr += 1
                }
            }
        }
        Ok((
            start_pos,
            self.ptr,
            text_element_type,
            TextElementTermination::EOF,
        ))
    }

    fn get_placeable(&mut self) -> Result<ast::Expression<S>> {
        panic!()
    }

    // *************************

    pub fn is_current_byte(&self, b: u8) -> bool {
        self.source.get_char(self.ptr) == Some(&b)
    }

    fn is_byte_at(&self, b: u8, pos: usize) -> bool {
        self.source.get_char(pos) == Some(&b)
    }

    fn skip_eol(&mut self) -> bool {
        match self.source.get_char(self.ptr) {
            Some(b'\n') => {
                self.ptr += 1;
                true
            }
            Some(b'\r') if self.is_byte_at(b'\n', self.ptr + 1) => {
                self.ptr += 2;
                true
            }
            _ => false,
        }
    }

    fn skip_blank_block(&mut self) -> usize {
        let mut count = 0;
        loop {
            let start = self.ptr;
            self.skip_blank_inline();
            if !self.skip_eol() {
                self.ptr = start;
                break;
            }
            count += 1;
        }
        count
    }

    fn skip_blank_inline(&mut self) -> usize {
        let start = self.ptr;
        while let Some(b' ') = self.source.get_char(self.ptr) {
            self.ptr += 1;
        }
        self.ptr - start
    }

    fn is_byte_pattern_continuation(&self, b: u8) -> bool {
        ![b'}', b'.', b'[', b'*'].contains(&b)
    }

    fn expect_byte(&mut self, b: u8) -> Result<()> {
        if !self.is_current_byte(b) {
            return error!(ErrorKind::ExpectedToken(b as char), self.ptr);
        }
        self.ptr += 1;
        Ok(())
    }
}
