use std::iter::FromIterator;
use std::time::Instant;
use crate::token::XmlTokenType::*;

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
pub struct XmlToken {
    pub token_type: XmlTokenType,
    pub content: String,
}

pub struct XmlTokenizer {
    pos: usize,
    data: Vec<char>,
}


pub fn default_tokenizer() -> XmlTokenizer {
    XmlTokenizer { data: vec![], pos: 0 }
}

//Tags
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


fn is_whitespace(c: char) -> bool {
    c == NEWLINE || c == TAB || c == SPACE || c == CARRIAGE_RETURN
}


impl XmlTokenizer {
    #[inline(always)]
    fn peek(&self) -> char {
        self.data[self.pos]
    }

    #[inline(always)]
    fn next(&mut self) -> char {
        let c = self.peek();
        self.pos += 1;
        c
    }

    fn has_next(&self) -> bool {
        self.pos < self.data.len()
    }

    fn expect(&mut self, expected: &str) {
        for c in expected.chars() {
            if c != self.next() {
                panic!("Expected {}, got something else", expected);
            }
        }
    }

    fn skip_whitespace(&mut self) -> String {
        self.next_token(|c| !is_whitespace(c))
    }

    fn is_upcoming(&self, slice: &str) -> bool {
        let mut i = 0;
        for c in slice.chars() {
            if c != self.data[self.pos + i] {
                return false;
            }
            i += 1;
        }
        return true;
    }

    fn next_token<F: Fn(char) -> bool>(&mut self, match_func: F) -> String {
        let start_pos = self.pos;
        while !match_func(self.peek()) {
            self.next();
        }
        String::from_iter(&self.data[start_pos..self.pos])
    }

    fn next_string_token(&mut self, token: &str) -> String {
        let start_pos = self.pos;
        while !self.is_upcoming(token) {
            self.next();
        }
        String::from_iter(&self.data[start_pos..self.pos])
    }

    pub fn tokenize(&mut self, xml: String) -> Vec<XmlToken> {
        // TODO multipeek iter
        // Initialize
        self.pos = 0;
        self.data = xml.chars().collect::<Vec<char>>();

        let mut tokens = vec![];
        self.tokenize_markup(&mut tokens);
        return tokens;
    }

    /// Identify comments, processing instructions and CDATA sections
    fn tokenize_markup(&mut self, tokens: &mut Vec<XmlToken>) {
        while self.has_next() {
            if self.is_upcoming(CLOSING_TAG_START) {
                self.tokenize_closing_tag(tokens);
            } else if self.is_upcoming(TAG_START) {
                self.tokenize_opening_tag(tokens);
            } else {
                let content = self.next_string_token(TAG_START);
                tokens.push(XmlToken {
                    token_type: Text,
                    content,
                })
            }
        }
        //println!("Done parsing");
        return;
    }

    fn tokenize_closing_tag(&mut self, tokens: &mut Vec<XmlToken>) {
        self.expect(CLOSING_TAG_START);

        self.skip_whitespace();

        let tag_name = self.next_string_token(TAG_END).trim_end().to_string();
        //println!("closing tag: {}", tag_name);
        self.skip_whitespace();

        self.expect(TAG_END);

        tokens.push(XmlToken {
            token_type: ClosingTag,
            content: tag_name,
        });
    }

    fn tokenize_opening_tag(&mut self, tokens: &mut Vec<XmlToken>) {
        self.expect(TAG_START);

        self.skip_whitespace();

        let tag_name = self.next_token(|c| is_whitespace(c) || c == TAG_END.chars().next().unwrap() || c == CLOSING_TAG.chars().next().unwrap());
        //println!("tag: {}", tag_name);

        self.skip_whitespace();
        let mut attributes = vec![];
        while !self.is_upcoming(TAG_END) && !self.is_upcoming(EMPTY_ELEMENT_TAG_END) {
            let (key, value) = self.tokenize_attribute();
            attributes.push(key);
            attributes.push(value);
            ;
        }
        let is_empty_element_tag = self.is_upcoming(EMPTY_ELEMENT_TAG_END);

        // Append attributes after tag token
        tokens.push(XmlToken {
            token_type: if is_empty_element_tag { EmptyElementTag } else { OpeningTag },
            content: tag_name,
        });
        tokens.append(&mut attributes);

        self.expect(if is_empty_element_tag { EMPTY_ELEMENT_TAG_END } else { TAG_END });
    }

    fn tokenize_attribute(&mut self) -> (XmlToken, XmlToken) {
        let attribute_key = self.next_string_token(ATTRIBUTE_KEY_VALUE_SEPARATOR).trim_end().to_string();
        self.expect(ATTRIBUTE_KEY_VALUE_SEPARATOR);
        self.skip_whitespace();
        self.expect(ATTRIBUTE_VALUE_START);
        let attribute_value = self.next_string_token(ATTRIBUTE_VALUE_END);
        self.expect(ATTRIBUTE_VALUE_END);
        self.skip_whitespace();
        (XmlToken { token_type: AttributeKey, content: attribute_key }, XmlToken { token_type: AttributeValue, content: attribute_value })
    }
}