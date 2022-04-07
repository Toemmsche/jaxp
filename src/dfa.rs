use std::collections::VecDeque;
use std::iter::FromIterator;
use std::str::Chars;
use std::thread::current;
use std::time::Instant;
use xmlparser::Stream;
use crate::CharStream;
use crate::dfa::XmlTokenType::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum XmlTokenType {
    None,
    Text,
    OpeningTag,
    ClosingTag,
    EmptyElementTag,
    CdataSection,
    Comment,
    ProcessingInstruction,
    AttributeKey,
    AttributeValue,
}

#[derive(Debug, Clone)]
pub struct XmlToken<'a> {
    pub token_type: XmlTokenType,
    pub content: &'a str,
}

pub struct DFA<'a> {
    pub(crate) cs: CharStream<'a>,
}

//Tags
const TAG_START_CHAR: u8 = b'<';
const TAG_END_CHAR: u8 = b'>';
const CLOSING_TAG_CHAR: u8 = b'/';

const TAG_START: &str = "<";
const TAG_END: &str = ">";
const CLOSING_TAG: &str = "/";
const CLOSING_TAG_START: &str = "</";
const EMPTY_ELEMENT_TAG_END: &str = "/>";

//Attributes
const ATTRIBUTE_KEY_VALUE_SEPARATOR: &str = "=";
const ATTRIBUTE_VALUE_START: &str = "\"";
const ATTRIBUTE_VALUE_END: &str = "\"";

// Whitespace characters
const NEWLINE: char = '\n';
const SPACE: char = ' ';
const TAB: char = '\t';
const CARRIAGE_RETURN: char = '\r';

#[inline(always)]
fn is_whitespace(c: char) -> bool {
    c == NEWLINE || c == TAB || c == SPACE || c == CARRIAGE_RETURN
}

impl<'a> DFA<'a> {
    pub fn tokenize(&'a mut self, xml: &str) -> Vec<XmlToken> {
        // Initialize
        let now = Instant::now();

        let tokens = self.tokenize_markup();

        let mut elapsed = now.elapsed();
        elapsed = now.elapsed();
        println!("Tokenize took: {:.2?}", elapsed);
        return tokens;
    }

    pub fn tokenize_markup(&'a mut self) -> Vec<XmlToken> {
        let cs = &mut self.cs;
        Self::tokenize_text(cs)
    }


    pub fn tokenize_opening_tag(cs: &mut CharStream<'a>) -> XmlToken<'a> {
        cs.expect(TAG_START);
        let start = cs.pos;
        cs.consume_name();
        let end = cs.pos;
        cs.skip_spaces();
        cs.expect(TAG_END);
        return XmlToken { token_type: OpeningTag, content: &cs.slice[start..end] };
    }

    pub fn tokenize_closing_tag(cs: &mut CharStream<'a>) -> XmlToken<'a> {
        cs.expect(CLOSING_TAG_START);
        let start = cs.pos;
        cs.consume_name();
        let end = cs.pos;
        cs.skip_spaces();
        cs.expect(TAG_END);
        return XmlToken { token_type: ClosingTag, content: &cs.slice[start..end] };
    }

    pub fn tokenize_text(cs: &mut CharStream<'a>) -> Vec<XmlToken<'a>> {
        let mut tokens = vec![];
        while cs.has_next_byte() {
            let start = cs.pos;
            cs.advance_until_byte(TAG_START_CHAR);
            let end = cs.pos;
            tokens.push(XmlToken { token_type: Text, content:  &cs.slice[start..end] });
            if cs.upcoming(CLOSING_TAG_START) {
                tokens.push(Self::tokenize_closing_tag(cs));
                return tokens;
            } else {
                tokens.push(Self::tokenize_opening_tag(cs));
            }
        }
        tokens
    }
}