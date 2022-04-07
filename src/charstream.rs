use std::char::ParseCharError;
use std::detect::__is_feature_detected::adx;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;
use std::str::{from_utf8, FromStr};

use crate::charstream::XmlTokenizeError::{IllegalToken, UnexpectedToken};
use crate::xmlchar::XmlChar;

pub struct CharStream<'a> {
    pub(crate) pos: usize,
    pub(crate) text: &'a str,
}

#[derive(Debug)]
pub struct PositionalError<'a> {
    pos: (usize, usize),
    // (row, column)
    env: &'a str,
    error: XmlTokenizeError<'a>,
}

#[derive(Debug)]
pub enum XmlTokenizeError<'a> {
    UnexpectedToken { expected: &'a str, actual: &'a str },
    IllegalToken { token: &'a str },
}

impl Display for XmlTokenizeError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        //TODO
        write!(f, "{:?}", self)
    }
}

impl Display for PositionalError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error at row {}, column {}: {}", self.pos.0, self.pos.1, self.error)
    }
}

pub type TextRange = (usize, usize);


impl<'a> CharStream<'a> {
    #[inline]
    pub fn range_is_empty(&self, range: TextRange) -> bool {
        range.0 >= range.1
    }
    #[inline]
    fn make_pos_error(&self, error: XmlTokenizeError<'a>) -> PositionalError<'a> {
        //TODO
        PositionalError { pos: (self.pos, self.pos), env: self.slice_from(0), error }
    }
    #[inline]
    fn upcoming_illegal_token(&mut self, num_bytes: usize) {
        let illegal_token = self.consume_n(num_bytes);
        panic!("{:?}", self.make_pos_error(IllegalToken { token: illegal_token }));
    }

    #[inline]
    fn unexpected_token(&mut self, expected: &str, actual: &str) {
        panic!("{:?}", self.make_pos_error(UnexpectedToken { expected, actual }));
    }

    #[inline]
    pub fn has_next(&self) -> bool {
        self.pos < self.text.len()
    }

    #[inline]
    pub fn peek_char(&mut self) -> char {
        self.slice((self.pos, self.text.len())).chars().next().unwrap()
    }

    #[inline]
    pub fn next_char(&mut self) -> char {
        let c = self.peek_char();
        self.pos += c.len_utf8();
        c
    }
    #[inline]
    pub fn skip_n(&mut self, i: usize) {
        self.pos += i;
    }
    #[inline]
    pub fn skip_spaces(&mut self) {
        while self.peek_char().is_xml_whitespace() {
            self.skip_n(1);
        };
    }
    #[inline]
    pub fn advance_n(&mut self, i: usize) -> TextRange {
        self.pos += i;
        (self.pos - i, self.pos)
    }
    #[inline]
    pub fn advance_until(&mut self, bytes: &str) -> TextRange {
        let from_pos = self.pos;
        while !self.upcoming(bytes) {
            self.skip_n(1);
        }
        (from_pos, self.pos)
    }
    #[inline]
    pub fn slice(&self, range: TextRange) -> &'a str {
        &self.text[range.0..range.1]
    }
    #[inline]
    pub fn slice_from(&self, from: usize) -> &'a str {
        &self.slice((from, self.pos))
    }
    #[inline]
    pub fn peek_n(&mut self, num_bytes: usize) -> &'a str {
        self.slice((self.pos, self.pos + num_bytes))
    }
    #[inline]
    pub fn consume_n(&mut self, num_bytes: usize) -> &'a str {
        let range = self.advance_n(num_bytes);
        self.slice(range)
    }

    #[inline]
    pub fn expect(&mut self, expected: &str) {
        let actual = self.consume_n(expected.len());
        if expected != actual {
            self.unexpected_token(expected, actual);
        }
    }
    #[inline]
    pub fn upcoming(&mut self, test: &str) -> bool {
        &self.text[self.pos..self.pos + test.len()] == test
    }

    #[inline]
    pub fn expect_name_start_char(&mut self) {
        let c = self.next_char();
        if !c.is_xml_name_start_char() {
            self.unexpected_token("NameStartChar", &c.to_string());
        }
    }

    /// Name ::= NameStartChar (NameChar)*
    /// [https://www.w3.org/TR/xml/#sec-common-syn]
    #[inline]
    pub fn consume_name(&mut self) -> TextRange {
        let from_pos = self.pos;
        self.expect_name_start_char();
        while self.peek_char().is_xml_name_char() {
            self.skip_n(1);
        }
        (from_pos, self.pos)
    }

    /// CharData ::= [^<&]* - ([^<&]* ']]>' [^<&]*)
    /// [https://www.w3.org/TR/xml/#syntax]
    #[inline]
    pub fn consume_character_data_until(&mut self, delimiter: char) -> TextRange {
        let from_pos = self.pos;
        let cdata_close_delimiter = "]]>";
        loop {
            let c = self.peek_char();
            if c == ']' {
                if self.upcoming(cdata_close_delimiter) {
                    self.upcoming_illegal_token(cdata_close_delimiter.len())
                }
            }
            if !c.is_xml_character_data_char() || c == delimiter {
                break;
            }
            self.skip_n(1);
        }
        (from_pos, self.pos)
    }

    /// CDSect ::= CDStart CData CDEnd
    /// CDStart	::= '<![CDATA['
    /// CData ::= (Char* - (Char* ']]>' Char*))
    /// CDEnd ::= ']]>'
    /// [https://www.w3.org/TR/xml/#sec-cdata-sect]
    #[inline]
    pub fn consume_cdata(&mut self) -> TextRange {
        let from_pos = self.pos;
        loop {
            let c = self.peek_char();
            if c == ']' {
                if self.upcoming("]]>") {
                    break;
                }
            } else if !c.is_xml_char() {
                break;
            }
            self.skip_n(1);
        }
        (from_pos, self.pos)
    }

    /// Comment ::= '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'
    /// [https://www.w3.org/TR/xml/#sec-comments]
    #[inline]
    pub fn consume_comment(&mut self) -> TextRange {
        let from_pos = self.pos;
        loop {
            let c = self.peek_char();
            if c == '-' {
                if self.upcoming("--") {
                    if self.upcoming("-->") {
                        break;
                    } else if self.upcoming("--->") {
                        // Last character cannot be a hyphen
                        self.upcoming_illegal_token(1);
                    } else {
                        // Double hypen is not allowed inside comments
                        self.upcoming_illegal_token(2);
                    }
                }
            } else if !c.is_xml_char() {
                break;
            }
            self.skip_n(1);
        }
        (from_pos, self.pos)
    }
}
