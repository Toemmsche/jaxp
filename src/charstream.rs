use core::result::Result::*;
use std::char::ParseCharError;
use std::fmt::{Debug, Display, Formatter};
use std::num::ParseIntError;
use std::ops::Range;
use std::str::{from_utf8, FromStr};
use std::string::ParseError;

use crate::token::XmlRangeToken;
use crate::xmlchar::XmlChar;
use crate::xmlerror::*;
use crate::xmlerror::XmlError::{DecodeReferenceError, IllegalToken, UnexpectedXmlToken, UnknownReference};

pub struct CharStream<'a> {
    pub(crate) pos: usize,
    pub(crate) text: &'a str,
}

impl Default for CharStream<'_> {
    fn default() -> Self {
        CharStream {
            pos: 0,
            text: "",
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextRange(pub(crate) usize, pub(crate) usize);


impl<'a> CharStream<'a> {
    #[inline]
    pub fn range_is_empty(&self, range: TextRange) -> bool {
        range.0 >= range.1
    }

    #[inline]
    pub fn has_next(&self) -> bool {
        self.pos < self.text.len()
    }

    #[inline]
    pub fn peek_char(&mut self) -> Result<char, XmlError> {
        let c = self.slice(TextRange(self.pos, self.text.len())).chars().next().unwrap();
        if !c.is_xml_char() {
            Err(IllegalToken {
                input: self.text.to_string(),
                range: TextRange(self.pos, self.pos + 1),
                expected: None,
            })
        } else {
            Ok(c)
        }
    }

    #[inline]
    pub fn next_char(&mut self) -> Result<char, XmlError> {
        let c = self.peek_char()?;
        self.pos += c.len_utf8();
        Ok(c)
    }

    #[inline]
    pub fn skip_n(&mut self, i: usize) {
        self.pos += i;
    }

    #[inline]
    pub fn skip_spaces(&mut self) -> Result<(), XmlError> {
        while self.peek_char()?.is_xml_whitespace() {
            self.skip_n(1);
        };
        Ok(())
    }

    #[inline]
    pub fn advance_n(&mut self, i: usize) -> TextRange {
        self.pos += i;
        TextRange(self.pos - i, self.pos)
    }

    #[inline]
    pub fn advance_until(&mut self, bytes: &str) -> TextRange {
        let from_pos = self.pos;
        while !self.upcoming(bytes) {
            self.skip_n(1);
        }
        TextRange(from_pos, self.pos)
    }

    #[inline]
    pub fn slice(&self, range: TextRange) -> &'a str {
        &self.text[range.0..range.1]
    }

    #[inline]
    pub fn slice_from(&self, from: usize) -> &'a str {
        &self.slice(TextRange(from, self.pos))
    }

    #[inline]
    pub fn peek_n(&mut self, num_bytes: usize) -> &'a str {
        self.slice(TextRange(self.pos, self.pos + num_bytes))
    }

    #[inline]
    pub fn consume_n(&mut self, num_bytes: usize) -> &'a str {
        let range = self.advance_n(num_bytes);
        self.slice(range)
    }

    #[inline]
    pub fn skip_over(&mut self, expected: &str) {
        self.skip_n(expected.len());
    }

    #[inline]
    pub fn expect(&mut self, expected: &str) -> Result<(), XmlError> {
        let actual = self.consume_n(expected.len());
        if expected != actual {
            Err(IllegalToken { input: self.text.to_string(), range: TextRange(self.pos - expected.len(), self.pos), expected: Some(expected.to_string()) })
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn expect_spaces(&mut self) -> Result<(), XmlError> {
        let from_pos = self.pos;
        self.skip_spaces()?;
        if from_pos == self.pos {
            Err(IllegalToken { input: self.text.to_string(), range: TextRange(from_pos, from_pos + 1), expected: Some("Any space".to_string()) })
        } else {
            Ok(())
        }
    }


    #[inline]
    pub fn upcoming(&mut self, test: &str) -> bool {
        if self.pos + test.len() > self.text.len() {
            return false;
        }
        &self.text[self.pos..self.pos + test.len()] == test
    }

    #[inline]
    pub fn expect_name_start_char(&mut self) -> Result<(), XmlError> {
        let c = self.next_char()?;
        if !c.is_xml_name_start_char() {
            Err(IllegalToken { input: self.text.to_string().to_string(), range: TextRange(self.pos - 1, self.pos), expected: Some("NameStartChar".to_string()) })
        } else {
            Ok(())
        }
    }

    /// Name ::= NameStartChar (NameChar)*
    /// [https://www.w3.org/TR/xml/#sec-common-syn]
    #[inline]
    pub fn consume_name(&mut self) -> Result<TextRange, XmlError> {
        let from_pos = self.pos;
        self.expect_name_start_char()?;
        while self.peek_char()?.is_xml_name_char() {
            self.skip_n(1);
        }
        Ok(TextRange(from_pos, self.pos))
    }

    pub fn decode_hex(reference: &'a str) -> Option<char> {
        let byte_vec: Vec<Result<u8, ParseIntError>> = (0..reference.len())
            .step_by(2)
            .map(|i| if i == reference.len() - 1 {
                u8::from_str_radix(&reference[i..], 16)
            } else {
                u8::from_str_radix(&reference[i..i + 2], 16)
            })
            .collect();
        // u32 can be constructed with 1-4 bytes
        return if byte_vec.len() > 4 || byte_vec.is_empty() {
            None
        } else {
            let mut res: u32 = 0;
            for i in 0..byte_vec.len() {
                res *= 256;
                match byte_vec[i] {
                    Err(_) => return None,
                    Ok(byte) => res += byte as u32
                };
            }
            let c = char::from_u32(res)?;
            if c.is_xml_char() {
                Some(c)
            } else {
                None
            }
        };
    }


    /// Consume a character reference.
    /// Apart from valid unicode character references, the short-hand definitions
    /// "&amp;" = &
    /// "&lt;" = <
    /// "&gt;"= >
    /// "&apos;" = '
    /// and "&quot;" = "
    /// are supported.
    ///
    /// CharRef	::= '&#' 0-9+ ';'| '&#x' 0-9a-fA-F+ ';'
    /// [https://www.w3.org/TR/xml/#dt-charref]
    /// [https://www.w3.org/TR/xml/#syntax]
    pub fn consume_character_reference(&mut self) -> Result<TextRange, XmlError> {
        let from_pos = self.pos;
        self.expect("&")?;
        if self.upcoming("#x") {
            self.expect("#x")?;

            // unicode char reference
            let char_hex_range = self.consume_chars_until(";")?;
            let char_hex = self.slice(char_hex_range);

            // decode character reference
            match Self::decode_hex(char_hex) {
                Some(c) => (),
                None => return Err(UnknownReference { input: self.text.to_string(), range: TextRange(from_pos, char_hex_range.1 + 1) })
            };
        } else if self.upcoming("#") {
            self.expect("#")?;

            // unicode char reference
            let code_point_range = self.consume_chars_until(";")?;
            let err = Err(UnknownReference { input: self.text.to_string(), range: TextRange(from_pos, code_point_range.1 + 1) });
            match u32::from_str(self.slice(code_point_range)) {
                Ok(codepoint) => {
                    match char::from_u32(codepoint) {
                        Some(c) => if !c.is_xml_char() {
                            return err;
                        },
                        None => return err
                    }
                }
                Err(_) => return err
            };
        } else {
            // short hand syntax
            let short_range = self.consume_chars_until(";")?;
            let short = self.slice(short_range);
            match short {
                "amp" | "lt" | "gt" | "apos" | "quot" => (), // all good
                _ => return Err(UnknownReference { input: self.text.to_string(), range: TextRange(from_pos, short_range.1 + 1) })
            }
        }
        self.expect(";")?;
        Ok(TextRange(from_pos, self.pos))
    }

    /// Consumes CharData, which cannot contain the literal & or < in addition to
    /// the CDATA section-close delimiter "]]>". However, the literal & can still
    /// be used to escape characters or define character references
    ///
    /// CharData ::= \[^<&\]* - (\[^<&\]* ']]>' \[^<&\]*)
    /// [https://www.w3.org/TR/xml/#syntax]
    #[inline]
    pub fn consume_character_data_until(&mut self, delimiter: char) -> Result<TextRange, XmlError> {
        let from_pos = self.pos;
        let cdata_close_delimiter = "]]>";
        loop {
            match self.peek_char()? {
                c if c == delimiter => break,
                ']' => if self.upcoming(cdata_close_delimiter) {
                    return Err(IllegalToken {
                        input: self.text.to_string(),
                        range: TextRange(self.pos, self.pos + cdata_close_delimiter.len()),
                        expected: None,
                    });
                },
                '&' => {
                    // TODO handle returned range
                    self.consume_character_reference()?;
                    continue;
                }
                _ => self.skip_n(1)
            }
        }
        Ok(TextRange(from_pos, self.pos))
    }


    #[inline]
    pub fn consume_chars_until(&mut self, delimiter: &str) -> Result<TextRange, XmlError> {
        let from_pos = self.pos;
        while !self.upcoming(delimiter) {
            self.peek_char()?; // checks for valid xml char
            self.skip_n(1);
        }
        Ok(TextRange(from_pos, self.pos))
    }

    /// Comment ::= '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'
    /// [https://www.w3.org/TR/xml/#sec-comments]
    #[inline]
    pub fn consume_comment(&mut self) -> Result<TextRange, XmlError> {
        let from_pos = self.pos;
        loop {
            if self.upcoming("--") {
                if self.upcoming("-->") {
                    break;
                } else if self.upcoming("--->") {
                    // Last character cannot be a hyphen
                    return Err(IllegalToken {
                        input: self.text.to_string(),
                        range: TextRange(self.pos, self.pos + 1),
                        expected: None,
                    });
                } else {
                    // Double hypen is not allowed inside comments
                    return Err(IllegalToken {
                        input: self.text.to_string(),
                        range: TextRange(self.pos, self.pos + 2),
                        expected: None,
                    });
                }
            }
            self.peek_char()?;
            self.skip_n(1);
        }
        Ok(TextRange(from_pos, self.pos))
    }
}