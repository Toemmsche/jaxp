use crate::error::XmlErrorRange;
use crate::textrange::TextRange;
use crate::token::XmlToken::*;

#[derive(Debug)]
pub enum XmlToken<'a> {
    Text(TextRange<'a>),
    StartTag(TextRange<'a>),
    EndTag(TextRange<'a>),
    CdataSection(TextRange<'a>),
    Comment(TextRange<'a>),
    ProcessingInstruction {
        target_range: TextRange<'a>,
        opt_value_range: Option<TextRange<'a>>,
    },
    Attribute {
        name_range: TextRange<'a>,
        value_range: TextRange<'a>,
    },

    // Prolog tokens
    XmlDeclaration {
        version_range: TextRange<'a>,
        opt_encoding_range: Option<TextRange<'a>>,
        opt_standalone_range: Option<TextRange<'a>>,
    },
    DocTypeDeclaration {
        name_range: TextRange<'a>,
        opt_system_entity_range: Option<TextRange<'a>>,
        opt_public_entity_range: Option<TextRange<'a>>,
    },
}

impl XmlToken<'_> {
    pub fn get_range(&self, input: &str) -> XmlErrorRange {
        match self {
            XmlDeclaration { version_range, .. }
            => XmlErrorRange {
                start: version_range.start,
                end: version_range.end,
                input: input.to_string(),
            },
            Text(range) |
            StartTag(range) |
            EndTag(range) |
            CdataSection(range) |
            Comment(range)
            => XmlErrorRange {
                start: range.start,
                end: range.end,
                input: input.to_string(),
            },
            ProcessingInstruction { target_range, .. }
            => XmlErrorRange {
                start: target_range.start,
                end: target_range.end,
                input: input.to_string(),
            },
            Attribute { name_range, .. } |
            DocTypeDeclaration { name_range, .. }
            => XmlErrorRange {
                start: name_range.start,
                end: name_range.end,
                input: input.to_string(),
            },
        }
    }
}