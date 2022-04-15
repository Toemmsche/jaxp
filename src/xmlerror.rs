use std::fmt::{Display, Formatter};

use crate::charstream::TextRange;
use crate::token::XmlRangeToken;
use crate::xmlerror::XmlError::{DecodeReferenceError, IllegalToken, UnexpectedXmlToken, UnknownReference};

#[derive(Debug)]
pub enum XmlError {
    UnexpectedXmlToken { input: String, token: XmlRangeToken },
    IllegalToken { input: String, range: TextRange, expected: Option<String> },
    UnknownReference { input: String, range: TextRange },
    DecodeReferenceError { input: String, range: TextRange },
}

impl Display for XmlError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Error: {:?}", self)
    }
}

impl XmlError {
    pub fn get_target(&self) -> String {
        match self {
            IllegalToken { input, range, .. } |
            UnknownReference { input, range } |
            DecodeReferenceError { input, range } => input[range.0..range.1].to_string(),
            UnexpectedXmlToken {input, token} => input[0..input.len()].to_string()
        }
    }
}