use std::str::FromStr;

use crate::chariter::CharIter;
use crate::error::XmlError;
use crate::error::XmlError::{IllegalToken, UnknownReference};
use crate::textrange::TextRange;
use crate::token::XmlToken;
use crate::token::XmlToken::*;
use crate::util;
use crate::xmlchar::XmlChar;

pub struct XmlTokenizer {}

impl Default for XmlTokenizer {
    fn default() -> Self {
        XmlTokenizer {}
    }
}


impl<'a> XmlTokenizer {
    pub fn tokenize(&mut self, xml: &'a str) -> Result<Vec<XmlToken<'a>>, XmlError> {
        let mut cs = CharIter { pos: 0, text: xml };
        return self.tokenize_markup(&mut cs);
    }

    /// Aka element content
    /// content	:= CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    fn tokenize_markup(&mut self, cs: &mut CharIter<'a>) -> Result<Vec<XmlToken<'a>>, XmlError> {
        // average token length of ~20 bytes
        let mut tokens = Vec::with_capacity(cs.text.len() / 20);
        while cs.has_next() {
            let text_range = Self::consume_character_data_until(cs, '<')?;
            if !text_range.is_empty() {
                tokens.push(Text(text_range));
            }
            if cs.test(b"</") {
                tokens.push(Self::tokenize_end_tag(cs)?);
            } else if cs.test(b"<!--") {
                tokens.push(Self::tokenize_comment(cs)?);
            } else if cs.test(b"<![CDATA[") {
                tokens.push(Self::tokenize_cdata_section(cs)?);
            } else if cs.test(b"<?") {
                tokens.push(Self::tokenize_processing_instruction(cs)?)
            } else {
                tokens.append(Self::tokenize_start_tag(cs)?.as_mut());
            }
        }
        Ok(tokens)
    }


    /// STag ::= '<' Name (S Attribute)* S? '>'///
    /// EmptyElemTag ::= '<' Name (S Attribute)* S? '/>
    /// [https://www.w3.org/TR/xml/#sec-starttags]

    fn tokenize_start_tag(cs: &mut CharIter<'a>) -> Result<Vec<XmlToken<'a>>, XmlError> {
        let mut tokens = vec![];

        //tag start has already been identified
        cs.skip_over(b"<");
        let name_range = Self::consume_name(cs)?;
        cs.skip_spaces()?;

        while !cs.test(b"/>") && !cs.test(b">") {
            tokens.push(Self::tokenize_attribute(cs)?);
            cs.skip_spaces();
        }

        // Empty Element Tag
        let is_empty_element_tag = cs.test(b"/>");
        if is_empty_element_tag {
            cs.expect_bytes(b"/>")?;
        } else {
            cs.expect_byte(b'>')?;
        }

        tokens.insert(0, StartTag(name_range));
        if is_empty_element_tag {
            // Create artificial end tag
            tokens.push(EndTag(name_range));
        }
        Ok(tokens)
    }

    /// ETag ::= '</' Name S? '>'
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    fn tokenize_end_tag(cs: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        cs.skip_over(b"</");
        let name_range = Self::consume_name(cs)?;
        cs.skip_spaces()?;
        cs.expect_byte(b'>')?;
        Ok(EndTag(name_range))
    }

    /// Attribute ::= Name Eq AttValue
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    fn tokenize_attribute(cs: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        // spaces have already been skipped
        let name_range = Self::consume_name(cs)?;
        cs.expect_byte(b'=')?;
        let used_quote = cs.next_xml_char()?;
        if !used_quote.is_xml_quote() {
            return Err(IllegalToken {
                range: cs.error_slice(cs.pos() - used_quote.len_utf8()..cs.pos()),
                expected: Some("Either \" or '".to_string()),
            });
        }
        let value_range = Self::consume_character_data_until(cs, used_quote)?;
        cs.advance_n(used_quote.len_utf8())?;
        Ok(Attribute { name_range, value_range })
    }

    /// CDSect ::= CDStart CData CDEnd
    /// CDStart	::= '<![CDATA['
    /// CData ::= (Char* - (Char* ']]>' Char*))
    /// CDEnd ::= ']]>'
    /// [https://www.w3.org/TR/xml/#sec-cdata-sect]
    fn tokenize_cdata_section(cs: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        cs.skip_over(b"<![CDATA[");
        let value_range = Self::consume_xml_chars_until(cs, b"]]>")?;
        cs.skip_over(b"]]>");
        Ok(CdataSection(value_range))
    }

    /// Comment ::= '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'
    /// [https://www.w3.org/TR/xml/#sec-comments]
    fn tokenize_comment(cs: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        cs.skip_over(b"<!--");
        let from_pos = cs.pos();
        loop {
            if cs.test(b"--") {
                if cs.test(b"-->") {
                    break;
                } else if cs.test(b"--->") {
                    // Last character cannot be a hyphen
                    return Err(IllegalToken {
                        range: cs.error_slice(cs.pos()..cs.pos() + 1),
                        expected: None,
                    });
                } else {
                    // Double hypen is not allowed inside comments
                    return Err(IllegalToken {
                        range: cs.error_slice(cs.pos()..cs.pos() + 2),
                        expected: None,
                    });
                }
            }
            cs.next_xml_char()?;
        }
        cs.skip_over(b"-->");
        Ok(Comment(cs.slice(from_pos..cs.pos())))
    }

    /// PI ::= '<?' PITarget (S (Char* - (Char* '?>' Char*)))? '?>'
    /// PITarget ::= Name - (('X' | 'x') ('M' | 'm') ('L' | 'l'))
    /// [https://www.w3.org/TR/xml/#sec-pi]
    fn tokenize_processing_instruction(cs: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        cs.skip_over(b"<?");
        let target_range = Self::consume_name(cs)?;
        cs.skip_spaces()?;
        // TODO handle XML in processing instruction

        let mut opt_value_range = None;
        if !cs.test(b"?>") {
            opt_value_range = Some(Self::consume_xml_chars_until(cs, b"?>")?);
        }

        cs.skip_over(b"?>");
        Ok(ProcessingInstruction { target_range, opt_value_range })
    }

    /// Name ::= NameStartChar (NameChar)*
    /// [https://www.w3.org/TR/xml/#sec-common-syn]
    pub fn consume_name(cs: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        let from_pos = cs.pos();
        let c = cs.next_xml_char()?;
        if !c.is_xml_name_start_char() {
            return Err(IllegalToken {
                range: cs.error_slice(from_pos..cs.pos()),
                expected: Some("Any Name start char".to_string()),
            });
        }
        loop {
            let c = cs.peek_xml_char()?;
            if c.is_xml_name_char() {
                cs.advance_n(c.len_utf8());
            } else {
                break;
            }
        }
        Ok(cs.slice(from_pos..cs.pos()))
    }

    /// Consumes CharData until a specified char is found.
    /// By the standard, CharData cannot contain the literal & or < in addition to
    /// the CDATA section-close delimiter "]]>".
    /// However, the literal & can still be used to escape characters or define character references.
    ///
    /// CharData ::= \[^<&\]* - (\[^<&\]* ']]>' \[^<&\]*)
    /// [https://www.w3.org/TR/xml/#syntax]
    fn consume_character_data_until(cs: &mut CharIter<'a>, delimiter: char) -> Result<TextRange<'a>, XmlError> {
        let from_pos = cs.pos();
        let cdata_close_delimiter = b"]]>";
        loop {
            match cs.peek_xml_char()? {
                c if c == delimiter => break,
                ']' => if cs.test(cdata_close_delimiter) {
                    return Err(IllegalToken {
                        range: cs.error_slice(cs.pos()..cs.pos() + cdata_close_delimiter.len()),
                        expected: Some("Not the CDATA section-close delimiter".to_string()),
                    });
                },
                '&' => {
                    // TODO handle returned range
                    Self::consume_character_reference(cs)?;
                    continue;
                }
                '<' => {
                    return Err(IllegalToken {
                        range: cs.error_slice(cs.pos()..cs.pos() + '<'.len_utf8()),
                        expected: Some("Not the less-than character".to_string()),
                    });
                }
                c => { cs.advance_n(c.len_utf8()); }
            }
        }
        Ok(cs.slice(from_pos..cs.pos()))
    }


    /// Consume any XML char until a specified byte slice is found
    fn consume_xml_chars_until(cs: &mut CharIter<'a>, delimiter: &[u8]) -> Result<TextRange<'a>, XmlError> {
        let from_pos = cs.pos();
        while !cs.test(delimiter) {
            cs.next_xml_char()?; // checks for valid XML char
        }
        Ok(cs.slice(from_pos..cs.pos()))
    }

    /// Consume a character reference.
    /// Apart from valid unicode character references, the short-hand definitions
    /// "&amp;" = &
    /// "&lt;" = <
    /// "&gt;"= >
    /// "&apos;" = '
    /// and "&quot;" = "
    /// are supported.
    ///
    /// CharRef	::= '&#' 0-9+ ';'| '&#x' 0-9a-fA-F+ ';'
    /// [https://www.w3.org/TR/xml/#dt-charref]
    /// [https://www.w3.org/TR/xml/#syntax]
    fn consume_character_reference(cs: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        let from_pos = cs.pos();
        cs.expect_byte(b'&')?;
        if cs.test(b"#x") {
            cs.skip_over(b"#x");

            // unicode char reference
            let char_hex_range = Self::consume_xml_chars_until(cs, b";")?;

            // decode character reference
            match util::decode_hex(char_hex_range.slice) {
                Some(_) => (),
                None => return Err(UnknownReference { range: cs.error_slice(from_pos..char_hex_range.end + 1) })
            };
        } else if cs.test(b"#") {
            cs.skip_over(b"#");

            // unicode char reference
            let code_point_range = Self::consume_xml_chars_until(cs, b";")?;
            let err = Err(UnknownReference { range: cs.error_slice(from_pos..code_point_range.end + 1) });
            match u32::from_str(code_point_range.slice) {
                Ok(codepoint) => {
                    match char::from_u32(codepoint) {
                        Some(c) => if !c.is_xml_char() {
                            return err;
                        },
                        None => return err
                    }
                }
                Err(_) => return err
            };
        } else {
            // short hand syntax
            let short_range = Self::consume_xml_chars_until(cs, b";")?;
            match short_range.slice {
                "amp" | "lt" | "gt" | "apos" | "quot" => (), // all good
                _ => return Err(UnknownReference { range: cs.error_slice(from_pos..short_range.end + 1) })
            }
        }
        cs.skip_over(b";");
        println!("{:?}", &cs.text[from_pos..cs.pos()]);
        Ok(cs.slice(from_pos..cs.pos()))
    }
}