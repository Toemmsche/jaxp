pub trait XmlChar {
    fn is_xml_char(&self) -> bool;

    fn is_xml_name_start_char(&self) -> bool;

    fn is_xml_name_char(&self) -> bool;

    fn is_xml_character_data_char(&self) -> bool;
}


pub trait XmlByte {
    fn is_xml_whitespace(&self) -> bool;
    fn is_xml_quote(&self) -> bool;
    fn is_xml_pubid_char(&self) -> bool;
}

impl XmlByte for u8 {
    /// [\[3\] S](https://www.w3.org/TR/xml/#NT-S)
    fn is_xml_whitespace(&self) -> bool {
        match self {
            b' ' | b'\n' | b'\t' | b'\r' => true,
            _ => false
        }
    }

    /// Deduced from  [\[10\] AttValue](https://www.w3.org/TR/xml/#NT-AttValue)
    fn is_xml_quote(&self) -> bool {
        match self {
            b'"' | b'\'' => true,
            _ => false
        }
    }

    /// PubidChar ::= #x20 | #xD | #xA | \[a-zA-Z0-9\] | \[-'()+,./:=?;!*#@$_%\]
    /// [\[13\] PubidChar](https://www.w3.org/TR/xml/#NT-PubidChar)
    fn is_xml_pubid_char(&self) -> bool {
        self.is_ascii_alphanumeric() || match self {
            0x20 |
            0xA |
            0xD |
            b'-' | b'\'' | b'(' |
            b')' | b'+' | b',' |
            b'.' | b'/' | b':' |
            b'=' | b'?' | b',' |
            b'!' | b'*' | b'#' |
            b'@' | b'$' | b'_' |
            b'%' => true,
            _ => false
        }
    }
}


impl XmlChar for char {

    /// [\[2\] Char](https://www.w3.org/TR/xml/#NT-Char)
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

    /// [\[4\] NameStartChar](https://www.w3.org/TR/xml/#NT-NameStartChar)
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

    /// [\[4a\] NameChar](https://www.w3.org/TR/xml/#NT-NameChar)
    fn is_xml_name_char(&self) -> bool {
        self.is_xml_name_start_char() || match self {
            '-' | '.' | '0'..='9' |
            '\u{B7}' |
            '\u{0300}'..='\u{036F}' |
            '\u{203F}'..='\u{2040}' => true,
            _ => false
        }
    }

    /// [\[14\] CharData](https://www.w3.org/TR/xml/#NT-CharData)
    fn is_xml_character_data_char(&self) -> bool {
        match self {
            '<' | '&' => false,
            _ => true
        }
    }
}