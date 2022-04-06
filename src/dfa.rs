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
    pub(crate) cs: CharStream<'a>
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
    pub fn tokenize(&mut self, xml: &str) -> Vec<XmlToken> {
        // Initialize
        let now = Instant::now();

        let mut tokens = vec![];
        self.tokenize_markup(&mut tokens);

        let mut elapsed = now.elapsed();
        elapsed = now.elapsed();
        println!("Tokenize took: {:.2?}", elapsed);
        return tokens;
    }

    pub fn tokenize_markup(&mut self, tokens: &mut Vec<XmlToken>) {
        self.tokenize_text(tokens);
    }


    pub fn tokenize_opening_tag(&mut self, tokens: &mut Vec<XmlToken>)  {
        self.cs.expect(TAG_START);
        let name = self.cs.consume_name();
        self.cs.skip_spaces();
        self.cs.expect(TAG_END);
        tokens.push(XmlToken{token_type: OpeningTag, content: name});
    }

    pub fn tokenize_closing_tag(&mut self, tokens: &mut Vec<XmlToken>) {
        self.cs.expect(CLOSING_TAG_START);
        let name = self.cs.consume_name();
        self.cs.skip_spaces();
        self.cs.expect(TAG_END);
        tokens.push(XmlToken{token_type: ClosingTag, content: name})
    }

    pub fn tokenize_text(&mut self, tokens: &mut Vec<XmlToken>) {
        while self.cs.has_next_byte() {
            let text_content = self.cs.advance_until_byte(TAG_START_CHAR);
            tokens.push(XmlToken{token_type: Text, content: text_content});
            if self.cs.upcoming(CLOSING_TAG_START) {
                self.tokenize_closing_tag(tokens);
                return;
            } else {
                self.tokenize_opening_tag(tokens);
            }
        }
    }
}