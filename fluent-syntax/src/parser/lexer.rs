use std::ops::Range;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(Range<usize>),
    EqSign,
    CommentSign,
    GroupCommentSign,
    ResourceCommentSign,
    Eol(usize), // one or more empty lines
    Eot,        // End Of Text
    Dot,
    MinusSign,
    Indent(usize),
    Text(Range<usize>),
    Junk(Range<usize>),
    OpenCurlyBraces,
    CloseCurlyBraces,
    OpenBraces,
    CloseBraces,
    DoubleQuote,
    Number(Range<usize>),
    SelectorArrow,
    Asterisk,
    OpenSquareBracket,
    CloseSquareBracket,
    Comma,
    Colon,
    Dollar,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LexerState {
    Resource,
    Pattern,
    PatternContinuation,
    PatternNewLine,
    Message,
    Comment,
    Expression,
}

#[derive(Debug, PartialEq)]
pub enum LexerError {
    Unknown,
}

type LexerResult = Result<Token, LexerError>;
type LexerOptionResult = Result<Option<Token>, LexerError>;

#[derive(Clone, Debug)]
pub struct Lexer<'l> {
    state: LexerState,
    source: &'l [u8],
    ptr: usize,
    entry_start: usize,
    stack: Vec<LexerState>,
}

#[derive(Debug, PartialEq)]
pub enum NextLine {
    TextContinuation(usize),
    Attribute,
    NewEntry,
    Eol(usize),
}

impl<'l> Lexer<'l> {
    pub fn new(source: &'l [u8]) -> Self {
        Lexer {
            state: LexerState::Resource,
            source,
            ptr: 0,
            entry_start: 0,
            stack: vec![],
        }
    }

    fn get_ident(&mut self) -> Token {
        let start = self.ptr;
        self.ptr += 1;
        while let Some(b) = self.source.get(self.ptr) {
            if !b.is_ascii_alphanumeric() && *b != b'-' && *b != b'_' {
                break;
            }
            self.ptr += 1;
        }
        Token::Identifier(start..self.ptr)
    }

    fn tokenize_message(&mut self) -> LexerOptionResult {
        match self.first_after_inline() {
            Some(b'=') => {
                self.ptr += 1;
                self.state = LexerState::Pattern;
                Ok(Some(Token::EqSign))
            }
            Some(b'.') => {
                self.ptr += 1;
                Ok(Some(Token::Dot))
            }
            Some(b'a'..=b'z') | Some(b'A'..=b'Z') => {
                let ident = self.get_ident();
                Ok(Some(ident))
            }
            None => Ok(None),
            _ => Err(LexerError::Unknown),
        }
    }

    fn tokenize_resource(&mut self) -> LexerOptionResult {
        self.entry_start = self.ptr;
        if let Some(b) = self.source.get(self.ptr) {
            match b {
                b'\r' if self.source.get(self.ptr + 1) == Some(&b'\n') => {
                    self.ptr += 2;
                    Ok(Some(Token::Eol(1)))
                }
                b'\n' => {
                    self.ptr += 1;
                    Ok(Some(Token::Eol(1)))
                }
                b'a'..=b'z' | b'A'..=b'Z' => {
                    let ident = self.get_ident();
                    self.state = LexerState::Message;
                    Ok(Some(ident))
                }
                b'#' => {
                    let sigil;
                    self.ptr += 1;
                    if let Some(b'#') = self.source.get(self.ptr) {
                        self.ptr += 1;
                        if let Some(b'#') = self.source.get(self.ptr) {
                            self.ptr += 1;
                            sigil = Token::ResourceCommentSign;
                        } else {
                            sigil = Token::GroupCommentSign;
                        }
                    } else {
                        sigil = Token::CommentSign;
                    }
                    match self.source.get(self.ptr) {
                        Some(b' ') => {
                            self.ptr += 1;
                            self.state = LexerState::Comment;
                            Ok(Some(sigil))
                        }
                        Some(b'\r') if self.source.get(self.ptr + 1) == Some(&b'\n') => {
                            self.ptr += 1;
                            Ok(Some(sigil))
                        }
                        Some(b'\n') => Ok(Some(sigil)),
                        _ => Err(LexerError::Unknown),
                    }
                }
                b'-' => {
                    self.ptr += 1;
                    self.state = LexerState::Message;
                    Ok(Some(Token::MinusSign))
                }
                _ => Err(LexerError::Unknown),
            }
        } else {
            Ok(None)
        }
    }

    fn skip_inline_ws(&mut self) {
        while let Some(b' ') = self.source.get(self.ptr) {
            self.ptr += 1;
        }
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.source.get(self.ptr) {
            if b != &b' ' && b != &b'\n' {
                break;
            }
            self.ptr += 1;
        }
    }

    fn first_after_inline(&mut self) -> Option<u8> {
        loop {
            match self.source.get(self.ptr) {
                Some(b' ') => {
                    self.ptr += 1;
                }
                b => {
                    return b.copied();
                }
            }
        }
    }

    fn tokenize_pattern(&mut self) -> LexerOptionResult {
        while self.source.get(self.ptr) == Some(&b' ') {
            self.ptr += 1;
        }
        match self.source.get(self.ptr) {
            Some(b'\r') if self.source.get(self.ptr + 1) == Some(&b'\n') => {
                self.ptr += 2;
                self.state = LexerState::PatternNewLine;
                return self.tokenize_pattern_new_line();
            }
            Some(b'\n') => {
                self.ptr += 1;
                self.state = LexerState::PatternNewLine;
                return self.tokenize_pattern_new_line();
            }
            Some(b'{') => {
                self.ptr += 1;
                self.skip_ws();
                self.state = LexerState::Expression;
                if !self.stack.is_empty() {
                    self.stack.push(LexerState::Pattern);
                }
                return Ok(Some(Token::OpenCurlyBraces));
            }
            Some(b'}') => {
                return Err(LexerError::Unknown);
            }
            None => {
                self.state = LexerState::Message;
                return Ok(None);
            }
            _ => {
                self.state = LexerState::PatternContinuation;
                return self.tokenize_pattern_continuation();
            }
        }
    }

    fn tokenize_pattern_continuation(&mut self) -> LexerOptionResult {
        let start = self.ptr;
        let end;
        loop {
            match self.source.get(self.ptr) {
                Some(b'\r') if self.source.get(self.ptr + 1) == Some(&b'\n') => {
                    end = self.ptr;
                    self.ptr += 1;
                    break;
                }
                Some(b'\n') => {
                    self.state = LexerState::PatternNewLine;
                    self.ptr += 1;
                    end = self.ptr;
                    break;
                }
                Some(b'{') => {
                    end = self.ptr;
                    if start != end {
                        break;
                    } else {
                        self.ptr += 1;
                        self.skip_ws();
                        self.state = LexerState::Expression;
                        if !self.stack.is_empty() {
                            self.stack.push(LexerState::Pattern);
                        }
                        return Ok(Some(Token::OpenCurlyBraces));
                    }
                }
                Some(b'}') => {
                    return Err(LexerError::Unknown);
                }
                None => {
                    end = self.ptr;
                    if start != end {
                        break;
                    } else {
                        self.ptr += 1;
                        self.skip_inline_ws();
                        self.state = LexerState::Message;
                        return Ok(Some(Token::Eot));
                    }
                }
                Some(_) => {
                    self.ptr += 1;
                }
            }
        }

        return Ok(Some(Token::Text(start..end)));
    }

    fn tokenize_pattern_new_line(&mut self) -> LexerOptionResult {
        let line_start = self.ptr;
        match self.first_after_inline() {
            Some(b'.') => {
                self.state = LexerState::Message;
                return Ok(Some(Token::Eot));
            }
            Some(b'\r') if self.source.get(self.ptr + 1) == Some(&b'\n') => {
                self.ptr += 2;
                return Ok(Some(Token::Eol(1)));
            }
            Some(b'\n') => {
                self.ptr += 1;
                return Ok(Some(Token::Eol(1)));
            }
            Some(b'{') => {
                let indent = self.ptr - line_start;
                self.state = LexerState::PatternContinuation;
                return Ok(Some(Token::Indent(indent)));
            }
            Some(b'*') => {
                self.state = self.stack.pop().unwrap_or(LexerState::Expression);
                return Ok(Some(Token::Eot));
            }
            Some(b'[') => {
                self.state = self.stack.pop().unwrap_or(LexerState::Expression);
                return Ok(Some(Token::Eot));
            }
            Some(b'}') => {
                self.state = self.stack.pop().unwrap_or(LexerState::Expression);
                return Ok(Some(Token::Eot));
            }
            None => {
                self.state = LexerState::Message;
                return Ok(Some(Token::Eot));
            }
            Some(_) => {
                let indent = self.ptr - line_start;
                if indent > 0 {
                    self.state = LexerState::PatternContinuation;
                    Ok(Some(Token::Indent(indent)))
                } else {
                    if self.stack.is_empty() {
                        self.state = LexerState::Resource;
                        Ok(Some(Token::Eot))
                    } else {
                        self.state = LexerState::Resource;
                        self.stack.clear();
                        Err(LexerError::Unknown)
                    }
                }
            }
        }
    }

    fn tokenize_expression(&mut self) -> LexerResult {
        match self.source.get(self.ptr) {
            Some(b'0'..=b'9') => {
                let start = self.ptr;
                self.ptr += 1;
                while let Some(b'0'..=b'9') = self.source.get(self.ptr) {
                    self.ptr += 1;
                }
                if self.source.get(self.ptr) == Some(&b'.') {
                    self.ptr += 1;
                    let mut has_decimal = false;
                    while let Some(b'0'..=b'9') = self.source.get(self.ptr) {
                        has_decimal = true;
                        self.ptr += 1;
                    }
                    if !has_decimal {
                        return Err(LexerError::Unknown);
                    }
                }
                let end = self.ptr;
                self.skip_ws();
                Ok(Token::Number(start..end))
            }
            Some(b'a'..=b'z') | Some(b'A'..=b'Z') => {
                let ident = self.get_ident();
                self.skip_ws();
                Ok(ident)
            }
            Some(b'.') => {
                self.ptr += 1;
                Ok(Token::Dot)
            }
            Some(b'-') if Some(&b'>') == self.source.get(self.ptr + 1) => {
                self.ptr += 2;
                self.skip_inline_ws();
                if self.source.get(self.ptr) == Some(&b'\n') {
                    self.skip_ws();
                    Ok(Token::SelectorArrow)
                } else {
                    Err(LexerError::Unknown)
                }
            }
            Some(b'-') => {
                self.ptr += 1;
                Ok(Token::MinusSign)
            }
            Some(b'(') => {
                self.ptr += 1;
                self.skip_ws();
                Ok(Token::OpenBraces)
            }
            Some(b',') => {
                self.ptr += 1;
                self.skip_ws();
                Ok(Token::Comma)
            }
            Some(b')') => {
                self.ptr += 1;
                self.skip_inline_ws();
                Ok(Token::CloseBraces)
            }
            Some(b'$') => {
                self.ptr += 1;
                Ok(Token::Dollar)
            }
            Some(b'"') => {
                self.ptr += 1;
                let start = self.ptr;
                while let Some(b) = self.source.get(self.ptr) {
                    match b {
                        b'\\' => match self.source.get(self.ptr + 1) {
                            Some(b'\\') => self.ptr += 2,
                            Some(b'{') => self.ptr += 2,
                            Some(b'"') => self.ptr += 2,
                            Some(b'u') => {
                                self.ptr += 2;
                                self.skip_unicode_escape_sequence(4)?;
                            }
                            Some(b'U') => {
                                self.ptr += 2;
                                self.skip_unicode_escape_sequence(6)?;
                            }
                            _ => {
                                return Err(LexerError::Unknown);
                            }
                        },
                        b'"' => {
                            let end = self.ptr;
                            self.ptr += 1;
                            self.skip_inline_ws();
                            return Ok(Token::Text(start..end));
                        }
                        b'\n' => {
                            self.ptr += 1;
                            break;
                        }
                        _ => {
                            self.ptr += 1;
                        }
                    }
                }
                Err(LexerError::Unknown)
            }
            Some(b' ') => {
                if let Some(b'}') = self.first_after_inline() {
                    self.ptr += 1;
                    self.state = LexerState::PatternContinuation;
                    Ok(Token::CloseCurlyBraces)
                } else {
                    Err(LexerError::Unknown)
                }
            }
            Some(b'*') => {
                self.ptr += 1;
                Ok(Token::Asterisk)
            }
            Some(b'[') => {
                self.ptr += 1;
                self.skip_ws();
                Ok(Token::OpenSquareBracket)
            }
            Some(b']') => {
                self.ptr += 1;
                self.state = LexerState::Pattern;
                self.stack.push(LexerState::Expression);
                Ok(Token::CloseSquareBracket)
            }
            Some(b':') => {
                self.ptr += 1;
                self.skip_ws();
                Ok(Token::Colon)
            }
            Some(b'{') => {
                self.stack.push(LexerState::Expression);
                self.ptr += 1;
                self.skip_ws();
                Ok(Token::OpenCurlyBraces)
            }
            Some(b'}') => {
                self.ptr += 1;
                self.state = self.stack.pop().unwrap_or(LexerState::PatternContinuation);
                Ok(Token::CloseCurlyBraces)
            }
            _ => Err(LexerError::Unknown),
        }
    }

    fn tokenize_comment(&mut self) -> LexerResult {
        let start = self.ptr;
        while let Some(b) = self.source.get(self.ptr) {
            self.ptr += 1;
            if &b'\n' == b {
                self.state = LexerState::Resource;
                return Ok(Token::Text(start..self.ptr - 1));
            }
            if &b'\r' == b && self.source.get(self.ptr) == Some(&b'\n') {
                self.ptr += 1;
                self.state = LexerState::Resource;
                return Ok(Token::Text(start..self.ptr - 2));
            }
        }
        self.state = LexerState::Resource;
        Ok(Token::Text(start..self.ptr))
    }

    #[inline]
    fn get_token(&mut self) -> LexerOptionResult {
        match self.state {
            LexerState::Resource => self.tokenize_resource(),
            LexerState::Pattern => self.tokenize_pattern(),
            LexerState::PatternContinuation => self.tokenize_pattern_continuation(),
            LexerState::PatternNewLine => self.tokenize_pattern_new_line(),
            LexerState::Message => self.tokenize_message(),
            LexerState::Comment => self.tokenize_comment().map(Option::Some),
            LexerState::Expression => self.tokenize_expression().map(Option::Some),
        }
    }

    fn collect_junk_range(&mut self, start: usize) -> Range<usize> {
        while let Some(b) = self.source.get(self.ptr) {
            if (b.is_ascii_alphabetic() || b == &b'#' || b == &b'-')
                && self.ptr > 1
                && self.source[self.ptr - 1] == b'\n'
            {
                break;
            }
            self.ptr += 1;
        }
        self.state = LexerState::Resource;
        start..self.ptr
    }

    #[inline]
    fn next(&mut self) -> Option<Token> {
        self.get_token().unwrap_or_else(|_| {
            let junk_range = self.collect_junk_range(self.ptr);
            Some(Token::Junk(junk_range))
        })
    }

    #[inline]
    pub fn try_next(&mut self) -> LexerOptionResult {
        self.get_token()
    }

    #[inline]
    pub fn expect(&mut self, token: Token) -> Result<(), LexerError> {
        if self.try_next()? != Some(token) {
            Err(LexerError::Unknown)
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn try_peek(&mut self) -> Result<Option<Token>, LexerError> {
        let ptr = self.ptr;
        let state = self.state;
        let entry_start = self.entry_start;
        self.get_token().map(move |token| {
            self.ptr = ptr;
            self.state = state;
            self.entry_start = entry_start;
            token
        })
    }

    pub fn get_state_snapshot(&self) -> (usize, LexerState, usize, Option<LexerState>){
        let ptr = self.ptr;
        let state = self.state;
        let entry_start = self.entry_start;
        let last_on_stack = self.stack.last().cloned();
        (ptr, state, entry_start, last_on_stack)
    }

    pub fn set_state(&mut self, input: (usize, LexerState, usize, Option<LexerState>)) {
        self.ptr = input.0;
        self.state = input.1;
        self.entry_start = input.2;
        if let Some(last_on_stack) = input.3 {
            self.stack.push(last_on_stack);
        }
    }

    #[inline]
    pub fn take_if(&mut self, token: Token) -> Result<bool, LexerError> {
        let state = self.get_state_snapshot();
        match self.get_token() {
            Ok(Some(t)) if t == token => Ok(true),
            _ => {
                self.set_state(state);
                Ok(false)
            }
        }
    }

    pub fn get_junk(&mut self) -> Range<usize> {
        self.collect_junk_range(self.entry_start)
    }

    fn skip_unicode_escape_sequence(&mut self, length: usize) -> Result<(), LexerError> {
        let start = self.ptr;
        for _ in 0..length {
            match self.source.get(self.ptr) {
                Some(b) if b.is_ascii_hexdigit() => self.ptr += 1,
                _ => break,
            }
        }
        if self.ptr - start != length {
            return Err(LexerError::Unknown);
        }
        Ok(())
    }
}

impl<'l> Iterator for Lexer<'l> {
    type Item = Token;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
