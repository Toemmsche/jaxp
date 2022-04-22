use std::fmt::{Display, Formatter};

use crate::error::XmlError::*;

#[derive(Debug)]
pub struct XmlErrorRange {
    pub start: usize,
    pub end: usize,
    pub input: String,
}

#[derive(Debug)]
pub enum XmlError {
    //InternalError,
    NonMatchingTags { start_tag: XmlErrorRange, end_tag: XmlErrorRange },
    UnexpectedXmlToken { range: XmlErrorRange },
    IllegalToken { range: XmlErrorRange, expected: Option<String> },
    UnknownReference { range: XmlErrorRange },
    UnexpectedEndOfFile { input: String },
    // DecodeReferenceError { range: XmlErrorRange },
}

impl Display for XmlError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Error: {:?}", self)
    }
}

impl XmlError {
    pub fn get_target(&self) -> String {
        match self {
            // InternalError => "Internal Error".to_string(),
            UnexpectedEndOfFile { input } => input.to_string(),
            NonMatchingTags { start_tag, end_tag } => start_tag.input[end_tag.start..end_tag.end].to_string(),
            IllegalToken { range, .. } |
            UnexpectedXmlToken { range } |
            UnknownReference { range } => range.input[range.start..range.end].to_string(),
        }
    }
}