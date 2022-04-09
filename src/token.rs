use crate::charstream::TextRange;
use crate::token::XmlToken::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum XmlToken {
    Text(TextRange),
    StartTag(TextRange),
    EndTag(TextRange),
    EmptyElementTag(TextRange),
    CdataSection(TextRange),
    Comment(TextRange),
    ProcessingInstruction { target_range: TextRange, opt_value_range: Option<TextRange> },
    Attribute { name_range: TextRange, value_range: TextRange },
}


impl XmlToken {
    pub fn encompassing_range(&self) -> TextRange {
        match self {
            Text(range) | StartTag(range) | EndTag(range) |
            EmptyElementTag(range) | CdataSection(range) | Comment(range) => range.to_owned(),
            ProcessingInstruction { target_range, opt_value_range } => (target_range.0, opt_value_range.map_or(target_range.1, |ovr| ovr.1)),
            Attribute { name_range, value_range } => (name_range.0, value_range.1)
        }.to_owned()
    }
}