pub trait XmlChar {
    fn is_xml_char(&self) -> bool;

    fn is_xml_whitespace(&self) -> bool;

    fn is_xml_name_start_char(&self) -> bool;

    fn is_xml_name_char(&self) -> bool;

    fn is_xml_character_data_char(&self) -> bool;

    fn is_xml_quote(&self) -> bool;
}


impl XmlChar for char {
    /// Char ::= #x9 | #xA | #xD | #x20-#xD7FF | #xE000-#xFFFD | #x10000-#x10FFFF
    /// [https://www.w3.org/TR/xml/#charsets]

    fn is_xml_char(&self) -> bool {
        match self {
            '\u{9}' |
            '\u{A}' |
            '\u{D}' |
            '\u{20}'..='\u{D7FF}' |
            '\u{E000}'..='\u{FFFD}' |
            '\u{10000}'..='\u{10FFFF}' => true,
            _ => false
        }
    }

    /// S ::= (#x20 | #x9 | #xD | #xA)+
    /// [https://www.w3.org/TR/xml/#sec-common-syn]

    fn is_xml_whitespace(&self) -> bool {
        match self {
            ' ' | '\n' | '\t' | '\r' => true,
            _ => false
        }
    }

    /// NameStartChar ::= ":" | \[A-Z\] | "_" | \[a-z\] |
    /// \[#xC0-#xD6\] | \[#xD8-#xF6\] | \[#xF8-#x2FF | \[#x370-#x37D\] |
    /// \[#x37F-#x1FFF\] | \[#x200C-#x200D\] | \[#x2070-#x218F\] |
    /// \[#x2C00-#x2FEF\] | \[#x3001-#xD7FF\] | \[#xF900-#xFDCF\] |
    /// \[#xFDF0-#xFFFD\] | \[#x10000-#xEFFFF\]
    /// [https://www.w3.org/TR/xml/#sec-common-syn]

    fn is_xml_name_start_char(&self) -> bool {
        match self {
            ':' | 'A'..='Z' | '_' | 'a'..='z' |
            '\u{C0}'..='\u{D6}' |
            '\u{D8}'..='\u{F6}' |
            '\u{F8}'..='\u{2FF}' |
            '\u{370}'..='\u{37D}' |
            '\u{37F}'..='\u{1FFF}' |
            '\u{200C}'..='\u{200D}' |
            '\u{2070}'..='\u{218F}' |
            '\u{2C00}'..='\u{2FEF}' |
            '\u{3001}'..='\u{D7FF}' |
            '\u{F900}'..='\u{FDCF}' |
            '\u{FDF0}'..='\u{FFFD}' |
            '\u{10000}'..='\u{EFFFF}' => true,
            _ => false
        }
    }

    /// NameChar ::= NameStartChar | "-" | "." | 0-9 |
    /// #xB7 | #x0300-#x036F | [#x203F-#x2040]
    /// [https://www.w3.org/TR/xml/#sec-common-syn]

    fn is_xml_name_char(&self) -> bool {
        self.is_xml_name_start_char() || match self {
            '-' | '.' | '0'..='9' |
            '\u{B7}' |
            '\u{0300}'..='\u{036F}' |
            '\u{203F}'..='\u{2040}' => true,
            _ => false
        }
    }

    /// CharData ::= \[^<&\]* - (\[^<&\]* ']]>' \[^<&\]*)
    /// [https://www.w3.org/TR/xml/#syntax]
    fn is_xml_character_data_char(&self) -> bool {
        match self {
            '<' | '&' => false,
            _ => true
        }
    }

    /// AttValue ::= '"' ([^<&"] | Reference)* '"'| "'" ([^<&'] | Reference)* "'"
    /// Definition found [here](https://www.w3.org/TR/xml/#NT-AttValue)
    fn is_xml_quote(&self) -> bool {
        match self {
            '"' | '\'' => true,
            _ => false
        }
    }
}