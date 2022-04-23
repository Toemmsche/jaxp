use core::result::Result::*;
use std::ops::Range;

use crate::error::*;
use crate::error::XmlError::{IllegalToken, UnexpectedEndOfFile};
use crate::textrange::TextRange;
use crate::xmlchar::{XmlByte, XmlChar};

pub struct CharIter<'a> {
    pub(crate) pos: usize,
    pub(crate) text: &'a str,
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
        let byte = self.peek_byte()?;
        let c = if byte.is_ascii() {
            char::from(byte)
        } else {
            self.text[self.pos..self.text.len()].chars().next().unwrap()
        };

        if !c.is_xml_char() {
            return Err(IllegalToken {
                range: self.error_slice(self.pos..self.pos + c.len_utf8()),
                expected: None,
            });
        }
        Ok(c)
    }

    /// Get the current byte and advance the iterator by one.
    /// Does NOT check for char boundaries
    pub fn next_byte(&mut self) -> Result<u8, XmlError> {
        let byte = self.peek_byte()?;
        self.pos += 1;
        Ok(byte)
    }

    /// Get the current byte without advancing the iterator
    /// Does NOT check for char boundaries
    pub fn peek_byte(&self) -> Result<u8, XmlError> {
        if !self.has_next() {
            return Err(UnexpectedEndOfFile { input: self.text.to_string() });
        }
        Ok(self.text.as_bytes()[self.pos])
    }

    /// Advance the iterator by n and get the range of text that was skipped
    pub fn advance_n(&mut self, n: usize) -> Result<(), XmlError> {
        if !self.has_next() {
            return Err(UnexpectedEndOfFile { input: self.text.to_string() });
        }
        self.pos += n;
        Ok(())
    }

    /// Advance the iterator by the length of a byte slice
    pub fn skip_over(&mut self, expected: &[u8]) -> Result<(), XmlError> {
        self.advance_n(expected.len())
    }

    /// Advance the iterator while the current char is a whitespace
    pub fn skip_spaces(&mut self) -> Result<(), XmlError> {
        while self.peek_byte()?.is_xml_whitespace() {
            self.pos += 1; // every whitespace is one byte long
        };
        Ok(())
    }

    /// Test if a specified byte slice starts at the current iterator position
    pub fn test(&mut self, test: &[u8]) -> bool {
        self.pos + test.len() <= self.text.len() &&
            &self.text.as_bytes()[self.pos..self.pos + test.len()] == test
    }

    /// Test if a specified byte slice starts after skipping spaces
    pub fn test_after_spaces(&mut self, test: &[u8]) -> bool {
        let start_pos = self.pos;
        self.skip_spaces();
        let result = self.test(test);
        self.pos = start_pos;
        result
    }


    /// Test if a specified byte slice starts at the current iterator position, return an error if it doesn't.
    pub fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), XmlError> {
        if !self.test(expected) {
            return Err(IllegalToken {
                range: self.error_slice(self.pos..self.pos + expected.len()),
                expected: Some(String::from_utf8(Vec::from(expected)).unwrap()),
            });
        }

        self.pos += expected.len();
        Ok(())
    }

    /// Test if the current byte equals the expected, return an error if it doesn't.
    pub fn expect_byte(&mut self, expected: u8) -> Result<(), XmlError> {
        if self.peek_byte()? != expected {
            let c = self.peek_xml_char()?;
            return Err(IllegalToken { range: self.error_slice(self.pos..self.pos + c.len_utf8()), expected: Some(char::from(expected).to_string()) });
        }
        self.pos += 1;
        Ok(())
    }

    /// Like [skip_spaces](CharIter::skip_spaces) but throws and error if no space is skipped.
    pub fn expect_spaces(&mut self) -> Result<(), XmlError> {
        let from_pos = self.pos;
        self.skip_spaces()?;
        if from_pos == self.pos {
            let c = self.peek_xml_char()?;
            return Err(IllegalToken { range: self.error_slice(from_pos..from_pos + c.len_utf8()), expected: Some("Any space".to_string()) });
        }
        Ok(())
    }


    /// Create a TextRange using a text range and the underlying text.
    pub fn slice(&self, range: Range<usize>) -> TextRange<'a> {
        TextRange { start: range.start, end: range.end, slice: &self.text[range] }
    }

    /// Capture the text region that caused an error as an owned, heap-allocated string
    pub fn error_slice(&self, range: Range<usize>) -> XmlErrorRange {
        // avoid cutting unicode chars in half
        let from_pos = range.start;
        let mut end = range.end;
        while !self.text.is_char_boundary(end) {
            end += 1;
        }
        XmlErrorRange { start: range.start, end, input: self.text() }
    }
}