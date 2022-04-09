use std::fmt::{Display, Formatter};
use crate::token::XmlToken;

#[derive(Debug)]
pub struct PositionalError<'a> {
    pub(crate) pos: (usize, usize),
    // (row, column)
    pub(crate) env: &'a str,
    pub(crate) error: XmlTokenizeError<'a>,
}

impl<'a> PositionalError<'a> {

    #[inline]
    pub fn make_pos_error(slice: &'a str, pos: usize, error: XmlTokenizeError<'a>) -> PositionalError<'a> {
        //TODO
        PositionalError { pos: (pos, pos), env: &slice[0..pos], error }
    }
}

#[derive(Debug)]
pub enum XmlTokenizeError<'a> {
    UnknownToken { token: &'a XmlToken },
    UnexpectedToken { expected: &'a str, actual: &'a str },
    IllegalToken { token: &'a str },
}

impl Display for XmlTokenizeError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        //TODO
        write!(f, "{:?}", self)
    }
}

impl Display for PositionalError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error at row {}, column {}: {}", self.pos.0, self.pos.1, self.error)
    }
}