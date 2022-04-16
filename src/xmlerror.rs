use std::fmt::{Display, Formatter};

use crate::textrange::TextRange;
use crate::token::XmlToken;
use crate::xmlerror::XmlError::{DecodeReferenceError, IllegalToken, UnexpectedXmlToken, UnknownReference};

#[derive(Debug)]
pub struct XmlErrorRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) input: String,
}

#[derive(Debug)]
pub enum XmlError {
    InternalError,
    NonMatchingTags { start_tag: XmlErrorRange, end_tag: XmlErrorRange },
    UnexpectedXmlToken { token: XmlErrorRange },
    IllegalToken { range: XmlErrorRange, expected: Option<String> },
    UnknownReference { range: XmlErrorRange },
    DecodeReferenceError { range: XmlErrorRange },
}

impl Display for XmlError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Error: {:?}", self)
    }
}

impl XmlError {
    pub fn get_target(&self) -> String {
        match self {
            InternalError => "Internal Error".to_string(),
            IllegalToken { range, .. } |
            UnknownReference { range } |
            DecodeReferenceError { range } => range.input.to_string(),
            UnexpectedXmlToken { token } => format!("{:?}", token)
        }
    }
}