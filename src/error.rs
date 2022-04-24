use std::fmt::{Display, Formatter, Debug};

#[derive(Debug)]
pub struct XmlErrorPos {
    pub row: usize,
    pub col: usize
}

#[derive(Debug)]
pub enum XmlError {
    //InternalError,
    NonMatchingTags { start_tag: XmlErrorPos, end_tag: XmlErrorPos },
    UnexpectedXmlToken { pos: XmlErrorPos },
    IllegalToken { pos: XmlErrorPos, expected: Option<String> },
    UnknownReference { pos: XmlErrorPos },
    UnexpectedEndOfFile,
}

impl Display for XmlError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Error: {:?}", self)
    }
}
