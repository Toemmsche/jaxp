use crate::textrange::TextRange;
use crate::token::XmlToken::*;

#[derive(Debug)]
pub enum XmlToken<'a> {
    Text(TextRange<'a>),
    StartTag(TextRange<'a>),
    EndTag(TextRange<'a>),
    CdataSection(TextRange<'a>),
    Comment(TextRange<'a>),
    ProcessingInstruction { target_range: TextRange<'a>, opt_value_range: Option<TextRange<'a>> },
    Attribute { name_range: TextRange<'a>, value_range: TextRange<'a> },
}