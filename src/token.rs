use crate::textrange::TextRange;

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
    ParameterEntityReference(TextRange<'a>),
}