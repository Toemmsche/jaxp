use core::result::Result::*;
use std::char::ParseCharError;
use std::fmt::{Debug, Display, Formatter};
use std::num::ParseIntError;
use std::ops::Range;
use std::str::{from_utf8, FromStr};
use std::string::ParseError;

use xmlparser::XmlByteExt;

use crate::textrange::TextRange;
use crate::token::XmlToken;
use crate::xmlchar::XmlChar;
use crate::xmlerror::*;
use crate::xmlerror::XmlError::{DecodeReferenceError, IllegalToken, UnexpectedXmlToken, UnknownReference};

pub struct CharIter<'a> {
    pub(crate) pos: usize,
    pub(crate) text: &'a str,
}

impl Default for CharIter<'_> {
    fn default() -> Self {
        CharIter {
            pos: 0,
            text: "",
        }
    }
}

impl<'a> CharIter<'a> {
    /// Get the underlying text as an owned String
    pub fn text(&self) -> String {
        self.text.to_string()
    }

    /// Get the current position as an index in the underlying string slice
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// If the iterator has more elements
    pub fn has_next(&self) -> bool {
        self.pos < self.text.len()
    }

    /// Get the current character and advance the iterator by the length of that character-
    /// Throws an error if the character is not a valid XML char.
    pub fn next_xml_char(&mut self) -> Result<char, XmlError> {
        let c = self.peek_xml_char()?; // error check performed inside peek
        self.pos += c.len_utf8();
        Ok(c)
    }

    /// Get the current character without advancing the iterator.
    /// Throws an error if the character is not a valid XML char.
    pub fn peek_xml_char(&mut self) -> Result<char, XmlError> {
        let byte = self.peek_byte();
        let mut c: char = '\u{0}';
        if byte.is_ascii() {
            c = char::from(byte);
        } else {
            c = self.text[self.pos..self.text.len()].chars().next().unwrap();
        }
        if !c.is_xml_char() {
            Err(IllegalToken {
                range: self.error_slice(self.pos..self.pos + 1),
                expected: None,
            })
        } else {
            Ok(c)
        }
    }

    /// Get the current byte and advance the iterator by one.
    /// Does NOT check for char boundaries
    pub fn next_byte(&mut self) -> u8 {
        let byte = self.peek_byte();
        self.pos += 1;
        byte
    }

    /// Get the current byte without advancing the iterator
    /// Does NOT check for char boundaries
    pub fn peek_byte(&self) -> u8 {
        self.text.as_bytes()[self.pos]
    }

    /// Advance the iterator by n and get the range of text that was skipped
    pub fn advance_n(&mut self, n: usize) {
        self.pos += n;
    }

    /// Advance the iterator by the length of a byte slice
    pub fn skip_over(&mut self, expected: &[u8]) {
        self.advance_n(expected.len());
    }

    /// Advance the iterator while the current char is a whitespace
    pub fn skip_spaces(&mut self) -> Result<(), XmlError> {
        while self.peek_xml_char()?.is_xml_whitespace() {
            self.advance_n(1); // every whitespace is one byte long
        };
        Ok(())
    }

    /// Test if a specified byte slice starts at the current iterator position
    pub fn test(&mut self, test: &[u8]) -> bool {
        self.pos + test.len() <= self.text.len() &&
            &self.text.as_bytes()[self.pos..self.pos + test.len()] == test
    }


    /// Test if a specified byte slice starts at the current iterator position, return an error if it doesn't.
    pub fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), XmlError> {
        if !self.test(expected) {
            Err(IllegalToken {
                range: self.error_slice(self.pos..self.pos + expected.len()),
                expected: Some(String::from_utf8(Vec::from(expected)).unwrap()),
            })
        } else {
            self.advance_n(expected.len());
            Ok(())
        }
    }

    /// Test if the current byte equals the expected, return an error if it doesn't.
    pub fn expect_byte(&mut self, expected: u8) -> Result<(), XmlError> {
        if self.next_byte() != expected {
            Err(IllegalToken { range: self.error_slice(self.pos - 1..self.pos), expected: Some(expected.to_string()) })
        } else {
            Ok(())
        }
    }

    /// Like [skip_spaces](CharIter::skip_spaces) but throws and error if no space is skipped.
    pub fn expect_spaces(&mut self) -> Result<(), XmlError> {
        let from_pos = self.pos;
        self.skip_spaces()?;
        if from_pos == self.pos {
            Err(IllegalToken { range: self.error_slice(from_pos..from_pos + 1), expected: Some("Any space".to_string()) })
        } else {
            Ok(())
        }
    }

    /// Create a TextRange using a text range and the underlying text.
    pub fn slice(&self, range: Range<usize>) -> TextRange<'a> {
        TextRange { start: range.start, end: range.end, slice: &self.text[range] }
    }

    pub fn error_slice(&self, range: Range<usize>) -> XmlErrorRange {
        XmlErrorRange { start: range.start, end: range.end, input: self.text() }
    }
}