use super::ast;
use super::lexer::Token;
use super::lexer::{Lexer, LexerError};
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub enum ParserError {
    Unknown,
}

impl From<LexerError> for ParserError {
    fn from(_input: LexerError) -> Self {
        Self::Unknown
    }
}

type ParserResult<T> = Result<T, ParserError>;

pub struct Parser<'p> {
    source: &'p str,
    lexer: Lexer<'p>,
    body: Vec<ast::ResourceEntry<'p>>,
    last_comment: Option<ast::Comment<'p>>,
}

impl<'p> Parser<'p> {
    pub fn new(source: &'p str) -> Self {
        Parser {
            lexer: Lexer::new(source.as_bytes()),
            source,
            body: vec![],
            last_comment: None,
        }
    }

    pub fn parse(mut self) -> Result<ast::Resource<'p>, (ast::Resource<'p>, Vec<ParserError>)> {
        loop {
            match self.add_entry() {
                Ok(true) => {}
                Ok(false) => break,
                Err(_err) => {
                    let junk = self.lexer.get_junk();
                    if let Some(comment) = self.last_comment.take() {
                        self.body
                            .push(ast::ResourceEntry::Entry(ast::Entry::Comment(comment)));
                    }
                    self.body.push(ast::ResourceEntry::Junk(&self.source[junk]));
                }
            }
        }

        if let Some(comment) = self.last_comment.take() {
            self.body
                .push(ast::ResourceEntry::Entry(ast::Entry::Comment(comment)));
        }

        Ok(ast::Resource {
            body: self.body.into_boxed_slice(),
        })
    }

    fn add_entry(&mut self) -> Result<bool, ParserError> {
        match self.lexer.try_next()? {
            Some(Token::Identifier(r)) => {
                self.add_message(r)?;
                Ok(true)
            }
            Some(c @ Token::CommentSign)
            | Some(c @ Token::GroupCommentSign)
            | Some(c @ Token::ResourceCommentSign) => {
                if let Some(comment) = self.last_comment.take() {
                    self.body
                        .push(ast::ResourceEntry::Entry(ast::Entry::Comment(comment)));
                }
                self.add_comment(&c)?;
                Ok(true)
            }
            Some(Token::Eol(_)) => {
                if let Some(comment) = self.last_comment.take() {
                    self.body
                        .push(ast::ResourceEntry::Entry(ast::Entry::Comment(comment)));
                }
                Ok(true)
            }
            Some(Token::MinusSign) => {
                self.add_term()?;
                Ok(true)
            }
            Some(t) => panic!("Token: {:#?}", t),
            None => Ok(false),
        }
    }

    fn add_message(&mut self, id: Range<usize>) -> ParserResult<()> {
        let id = ast::Identifier {
            name: &self.source[id],
        };

        self.lexer.expect(Token::EqSign)?;

        let pattern = self.maybe_get_pattern()?;

        let mut attributes = vec![];
        let err = match self.get_attributes(&mut attributes) {
            Ok(()) => None,
            Err(err) => Some(err),
        };

        if pattern.is_none() && attributes.is_empty() {
            return Err(ParserError::Unknown);
        }

        self.body
            .push(ast::ResourceEntry::Entry(ast::Entry::Message(
                ast::Message {
                    id,
                    value: pattern,
                    attributes: attributes.into_boxed_slice(),
                    comment: self.last_comment.take(),
                },
            )));
        if let Some(err) = err {
            Err(err)
        } else {
            Ok(())
        }
    }

    fn add_term(&mut self) -> ParserResult<()> {
        let id = self.get_identifier()?;

        self.lexer.expect(Token::EqSign)?;

        let pattern = self.get_pattern()?;

        let mut attributes = vec![];
        let err = match self.get_attributes(&mut attributes) {
            Ok(()) => None,
            Err(err) => Some(err),
        };

        self.body
            .push(ast::ResourceEntry::Entry(ast::Entry::Term(ast::Term {
                id,
                value: pattern,
                attributes: attributes.into_boxed_slice(),
                comment: self.last_comment.take(),
            })));
        if let Some(err) = err {
            Err(err)
        } else {
            Ok(())
        }
    }

    fn maybe_get_pattern(&mut self) -> ParserResult<Option<ast::Pattern<'p>>> {
        // Arena?
        let mut pe = Vec::with_capacity(1);
        let mut indents = vec![];
        loop {
            match self.lexer.try_next()? {
                Some(Token::Text(r)) => {
                    pe.push(ast::PatternElement::TextElement(&self.source[r]));
                }
                Some(Token::Indent(i)) => match self.lexer.try_next()? {
                    Some(Token::Text(r)) => {
                        pe.push(ast::PatternElement::TextElement(&self.source[r.clone()]));
                        indents.push((i, Some(r), pe.len() - 1));
                    }
                    Some(Token::OpenCurlyBraces) => {
                        let exp = self.get_expression()?;
                        self.lexer.expect(Token::CloseCurlyBraces)?;
                        pe.push(ast::PatternElement::Placeable(exp));
                        indents.push((i, None, pe.len() - 1));
                    }
                    _ => return Err(ParserError::Unknown),
                },
                Some(Token::Eol(eols)) => {
                    if !pe.is_empty() {
                        for _ in 0..eols {
                            pe.push(ast::PatternElement::TextElement("\n"));
                        }
                    }
                }
                Some(Token::OpenCurlyBraces) => {
                    let exp = self.get_expression()?;
                    self.lexer.expect(Token::CloseCurlyBraces)?;
                    pe.push(ast::PatternElement::Placeable(exp));
                }
                Some(Token::Eot) => {
                    break;
                }
                _ => {
                    return Err(ParserError::Unknown);
                }
            }
        }
        if pe.is_empty() {
            Ok(None)
        } else {
            if !indents.is_empty() {
                let min_indent = indents.iter().map(|(i, _, _)| *i).min();
                if let Some(min_indent) = min_indent {
                    for (i, r, p) in indents {
                        let indent = i - min_indent;
                        match (unsafe { pe.get_unchecked_mut(p) }, r) {
                            (ast::PatternElement::TextElement(ref mut s), Some(ref r)) => {
                                *s = &self.source[r.start - indent..r.end];
                            }
                            (ast::PatternElement::Placeable(_), None) => {}
                            _ => unreachable!(),
                        }
                    }
                }
            }
            if pe.len() > 1 {
                while let Some(ast::PatternElement::TextElement("\n")) = pe.last() {
                    pe.pop();
                }
            }
            if let Some(ast::PatternElement::TextElement(ref mut s)) = pe.last_mut() {
                *s = s.trim_end();
            }
            Ok(Some(ast::Pattern {
                elements: pe.into_boxed_slice(),
            }))
        }
    }

    fn get_pattern(&mut self) -> ParserResult<ast::Pattern<'p>> {
        if let Some(pattern) = self.maybe_get_pattern()? {
            Ok(pattern)
        } else {
            Err(ParserError::Unknown)
        }
    }

    fn get_identifier(&mut self) -> ParserResult<ast::Identifier<'p>> {
        match self.lexer.try_next()? {
            Some(Token::Identifier(r)) => Ok(ast::Identifier {
                name: &self.source[r],
            }),
            _ => Err(ParserError::Unknown),
        }
    }

    fn get_attributes(&mut self, attributes: &mut Vec<ast::Attribute<'p>>) -> ParserResult<()> {
        let mut state = self.lexer.get_state_snapshot();
        while self.lexer.take_if(Token::Dot)? {
            match self.get_attribute() {
                Ok(attr) => attributes.push(attr),
                Err(err) => {
                    state.2 = state.0;
                    self.lexer.set_state(state);
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    fn get_attribute(&mut self) -> ParserResult<ast::Attribute<'p>> {
        let id = self.get_identifier()?;
        self.lexer.expect(Token::EqSign)?; // EqSign
        match self.maybe_get_pattern()? {
            Some(value) => Ok(ast::Attribute { id, value }),
            None => Err(ParserError::Unknown),
        }
    }

    fn add_comment(&mut self, token: &Token) -> ParserResult<()> {
        let mut pe = Vec::with_capacity(1);

        let result;
        loop {
            match self.add_comment_line(token, &mut pe) {
                Ok(true) => {}
                Ok(false) => {
                    result = Ok(());
                    break;
                }
                Err(err) => {
                    result = Err(err);
                    break;
                }
            }
        }
        let comment_type = match token {
            Token::CommentSign => ast::CommentType::Regular,
            Token::GroupCommentSign => ast::CommentType::Group,
            Token::ResourceCommentSign => ast::CommentType::Resource,
            _ => unreachable!(),
        };
        if comment_type == ast::CommentType::Regular {
            self.last_comment = Some(ast::Comment {
                comment_type,
                content: pe.into_boxed_slice(),
            });
        } else {
            self.body
                .push(ast::ResourceEntry::Entry(ast::Entry::Comment(
                    ast::Comment {
                        comment_type,
                        content: pe.into_boxed_slice(),
                    },
                )));
        }
        result
    }

    fn add_comment_line(&mut self, token: &Token, pe: &mut Vec<&'p str>) -> ParserResult<bool> {
        match self.lexer.try_next()? {
            Some(Token::Text(r)) => {
                pe.push(&self.source[r.start..r.end]);
            }
            Some(Token::Eol(_)) => {
                pe.push("");
            }
            None => {}
            _ => {
                return Err(ParserError::Unknown);
            }
        }
        match self.lexer.try_peek()? {
            Some(c @ Token::CommentSign)
            | Some(c @ Token::GroupCommentSign)
            | Some(c @ Token::ResourceCommentSign)
                if &c == token =>
            {
                self.lexer.try_next()?;
            }
            _ => {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn get_expression(&mut self) -> ParserResult<ast::Expression<'p>> {
        let inline_expression = self.get_inline_expression()?;

        if !self.lexer.take_if(Token::SelectorArrow)? {
            if let ast::InlineExpression::TermReference { ref attribute, .. } = inline_expression {
                if attribute.is_some() {
                    return Err(ParserError::Unknown);
                }
            }
            return Ok(ast::Expression::InlineExpression(inline_expression));
        }

        match inline_expression {
            ast::InlineExpression::MessageReference { ref attribute, .. } => {
                if attribute.is_none() {
                    return Err(ParserError::Unknown);
                } else {
                    return Err(ParserError::Unknown);
                }
            }
            ast::InlineExpression::TermReference { ref attribute, .. } => {
                if attribute.is_none() {
                    return Err(ParserError::Unknown);
                }
            }
            ast::InlineExpression::StringLiteral { .. }
            | ast::InlineExpression::NumberLiteral { .. }
            | ast::InlineExpression::VariableReference { .. }
            | ast::InlineExpression::FunctionReference { .. } => {}
            _ => {
                return Err(ParserError::Unknown);
            }
        };

        let variants = self.get_variants()?;
        Ok(ast::Expression::SelectExpression {
            selector: inline_expression,
            variants: variants.into_boxed_slice(),
        })
    }

    fn get_inline_expression(&mut self) -> ParserResult<ast::InlineExpression<'p>> {
        match self.lexer.try_next()? {
            Some(Token::MinusSign) => match self.lexer.try_next()? {
                Some(Token::Number(r)) => {
                    let num = self.get_number(r, true)?;
                    Ok(ast::InlineExpression::NumberLiteral {
                        value: &self.source[num],
                    })
                }
                Some(Token::Identifier(r)) => {
                    let id = ast::Identifier {
                        name: &self.source[r],
                    };
                    let attribute = if self.lexer.take_if(Token::Dot)? {
                        Some(self.get_identifier()?)
                    } else {
                        None
                    };
                    let arguments = self.get_call_arguments()?;
                    Ok(ast::InlineExpression::TermReference {
                        id,
                        attribute,
                        arguments,
                    })
                }
                _ => Err(ParserError::Unknown),
            },
            Some(Token::Dollar) => match self.lexer.try_next()? {
                Some(Token::Identifier(r)) => {
                    let id = ast::Identifier {
                        name: &self.source[r],
                    };
                    Ok(ast::InlineExpression::VariableReference { id })
                }
                _ => Err(ParserError::Unknown),
            },
            Some(Token::Identifier(r)) => {
                let id = ast::Identifier {
                    name: &self.source[r],
                };
                let attribute = if self.lexer.take_if(Token::Dot)? {
                    Some(self.get_identifier()?)
                } else {
                    None
                };
                let arguments = self.get_call_arguments()?;
                if arguments.is_some() {
                    if !id.name.bytes().all(|c| {
                        c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_' || c == b'-'
                    }) {
                        return Err(ParserError::Unknown);
                    }
                    Ok(ast::InlineExpression::FunctionReference { id, arguments })
                } else {
                    Ok(ast::InlineExpression::MessageReference { id, attribute })
                }
            }
            Some(Token::Number(r)) => {
                let num = self.get_number(r, false)?;
                Ok(ast::InlineExpression::NumberLiteral {
                    value: &self.source[num],
                })
            }
            Some(Token::Text(r)) => Ok(ast::InlineExpression::StringLiteral {
                value: &self.source[r],
            }),
            Some(Token::OpenCurlyBraces) => {
                let exp = self.get_expression()?;
                self.lexer.expect(Token::CloseCurlyBraces)?;
                Ok(ast::InlineExpression::Placeable {
                    expression: Box::new(exp),
                })
            }
            _ => Err(ParserError::Unknown),
        }
    }

    fn get_number(&mut self, r: Range<usize>, minus: bool) -> ParserResult<Range<usize>> {
        let mut result = r;
        if minus {
            result.start -= 1;
        }

        Ok(result)
    }

    fn get_call_arguments(&mut self) -> ParserResult<Option<ast::CallArguments<'p>>> {
        if self.lexer.take_if(Token::OpenBraces)? {
            let mut positional = vec![];
            let mut named = vec![];
            let mut argument_names = vec![];

            loop {
                if self.lexer.take_if(Token::CloseBraces)? {
                    break;
                }

                let expr = self.get_inline_expression()?;

                match expr {
                    ast::InlineExpression::MessageReference {
                        ref id,
                        attribute: None,
                    } => {
                        if self.lexer.take_if(Token::Colon)? {
                            if argument_names.contains(&id.name.to_owned()) {
                                return Err(ParserError::Unknown);
                            }
                            let val = self.get_inline_expression()?;
                            argument_names.push(id.name.to_owned());
                            named.push(ast::NamedArgument {
                                name: ast::Identifier { name: id.name },
                                value: val,
                            });
                        } else {
                            if !argument_names.is_empty() {
                                return Err(ParserError::Unknown);
                            }
                            positional.push(expr);
                        }
                    }
                    _ => {
                        if !argument_names.is_empty() {
                            return Err(ParserError::Unknown);
                        }
                        positional.push(expr);
                    }
                }

                if !self.lexer.take_if(Token::Comma)? {
                    if self.lexer.take_if(Token::CloseBraces)? {
                        break;
                    }
                }
            }

            Ok(Some(ast::CallArguments {
                positional: positional.into_boxed_slice(),
                named: named.into_boxed_slice(),
            }))
        } else {
            Ok(None)
        }
    }

    fn get_variants(&mut self) -> ParserResult<Vec<ast::Variant<'p>>> {
        let mut variants = vec![];
        let mut has_default = false;

        loop {
            let default = self.lexer.take_if(Token::Asterisk)?;

            if self.lexer.take_if(Token::OpenSquareBracket)? {
                let key = self.get_variant_key()?;
                self.lexer.expect(Token::CloseSquareBracket)?;
                let value = self.get_pattern()?;
                variants.push(ast::Variant {
                    key,
                    value,
                    default,
                });
            } else {
                break;
            }
        }
        Ok(variants)
    }

    fn get_variant_key(&mut self) -> ParserResult<ast::VariantKey<'p>> {
        match self.lexer.try_next()? {
            Some(Token::Identifier(r)) => Ok(ast::VariantKey::Identifier {
                name: &self.source[r],
            }),
            Some(Token::Number(r)) => Ok(ast::VariantKey::NumberLiteral {
                value: &self.source[r],
            }),
            _ => panic!(),
        }
    }
}
