use std::str::FromStr;

use crate::chariter::CharIter;
use crate::error::XmlError;
use crate::error::XmlError::{IllegalToken, UnknownReference};
use crate::textrange::TextRange;
use crate::token::XmlToken;
use crate::token::XmlToken::*;
use crate::util;
use crate::xmlchar::{XmlByte, XmlChar};

pub struct XmlTokenizer {}

impl Default for XmlTokenizer {
    fn default() -> Self {
        XmlTokenizer {}
    }
}


impl<'a> XmlTokenizer {
    pub fn tokenize(&mut self, xml: &'a str) -> Result<Vec<XmlToken<'a>>, XmlError> {
        let mut cs = CharIter { pos: 0, text: xml };

        // tokenize prolog

        return Self::tokenize_document(&mut cs);
    }

    /// [\[1\] document](https://www.w3.org/TR/xml/#NT-document)
    fn tokenize_document(cs: &mut CharIter<'a>) -> Result<Vec<XmlToken<'a>>, XmlError> {
        let mut tokens = Self::tokenize_prolog(cs)?;
        tokens.append(&mut Self::tokenize_content(cs)?);
        return Ok(tokens);
    }

    /// [\[22\] prolog](https://www.w3.org/TR/xml/#NT-prolog)
    fn tokenize_prolog(cs: &mut CharIter<'a>) -> Result<Vec<XmlToken<'a>>, XmlError> {
        if cs.test(b"<?xml") {
            return Ok(vec![Self::tokenize_xml_declaration(cs)?]);
        }
        Ok(vec![])
    }

    /// [\[23\] XMLDecl](https://www.w3.org/TR/xml/#NT-XMLDecl)
    fn tokenize_xml_declaration(cs: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        cs.skip_over(b"<?xml");
        let xml_decl_end_delim = b"?>";
        let version_info_range = Self::consume_version_info(cs)?;
        let mut encoding_declaration_range = None;
        let mut standalone_document_declaration_range = None;
        if cs.test_after_expected_space(b"encoding") {
            encoding_declaration_range = Some(Self::consume_encoding_declaration(cs)?);
        }
        if cs.test_after_expected_space(b"standalone") {
            standalone_document_declaration_range = Some(Self::consume_standalone_document_declaration(cs)?);
        }
        cs.skip_spaces()?;
        cs.expect_bytes(xml_decl_end_delim)?;
        Ok(XmlDeclaration {
            version_range: version_info_range,
            opt_encoding_range: encoding_declaration_range,
            opt_standalone_range: standalone_document_declaration_range,
        })
    }


    /// [\[32\] SDDecl](https://www.w3.org/TR/xml/#NT-SDDecl)
    fn consume_standalone_document_declaration(cs: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        cs.expect_spaces()?;
        cs.expect_bytes(b"standalone")?;
        Self::expect_eq(cs)?;
        let used_quote = Self::consume_quote(cs)?;

        let start_pos = cs.pos();
        if cs.test(b"yes") {
            cs.skip_over(b"yes");
        } else if cs.test(b"no") {
            cs.skip_over(b"no");
        } else {
            return Err(
                IllegalToken {
                    range: cs.error_slice(start_pos..cs.pos() + 3),
                    expected: Some("yes or no".to_string()),
                }
            );
        }
        let end_pos = cs.pos();
        cs.expect_byte(used_quote)?;
        return Ok(cs.slice(start_pos..end_pos));
    }


    /// [\[80\] EncodingDecl](https://www.w3.org/TR/xml/#NT-EncodingDecl)
    fn consume_encoding_declaration(cs: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        cs.expect_spaces()?;
        cs.expect_bytes(b"encoding")?;
        Self::expect_eq(cs)?;
        let used_quote = Self::consume_quote(cs)?;

        let range = Self::consume_encoding_name(cs)?;
        println!("{:?}", range);
        cs.expect_byte(used_quote)?;
        return Ok(range);
    }

    /// [\[81\] EncName](https://www.w3.org/TR/xml/#NT-VersionNum)
    fn consume_encoding_name(cs: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        let start_pos = cs.pos();
        /* Encoding name contains only Latin characters */
        let byte = cs.next_byte()?;
        if !byte.is_ascii_alphabetic() {
            return Err(IllegalToken {
                range: cs.error_slice(start_pos..cs.pos()),
                expected: Some("Any latin letter".to_string()),
            });
        }
        while match cs.peek_byte()? {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'| b'.' | b'_' | b'-' => {
               cs.advance_n(1);
                true
            }
            _ => false
        } {}
        Ok(cs.slice(start_pos..cs.pos()))
    }


    /// [\[24\] VersionInfo](https://www.w3.org/TR/xml/#NT-VersionInfo)
    fn consume_version_info(cs: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        cs.expect_spaces()?;
        cs.expect_bytes(b"version")?;
        Self::expect_eq(cs)?;
        let used_quote = Self::consume_quote(cs)?;

        let range = Self::consume_version_num(cs)?;
        cs.expect_byte(used_quote)?;
        return Ok(range);
    }

    /// [\[26\] VersionNUm](https://www.w3.org/TR/xml/#NT-VersionNum)
    fn consume_version_num(cs: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        let start_pos = cs.pos();
        cs.expect_bytes(b"1.")?;
        // TODO: remove redundant xml_char check
        while cs.peek_xml_char()?.is_ascii_digit() {
            cs.next_xml_char()?;
        }
        Ok(cs.slice(start_pos..cs.pos()))
    }

    /// content	:= CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*
    /// [content](https://www.w3.org/TR/xml/#sec-starttags)
    fn tokenize_content(cs: &mut CharIter<'a>) -> Result<Vec<XmlToken<'a>>, XmlError> {
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

        while !cs.test(b"/>") && !cs.test(b">")  && !cs.test_after_expected_space(b"/>") && !cs.test_after_expected_space(b">") {
            cs.expect_spaces()?;
            tokens.push(Self::tokenize_attribute(cs)?);
        }

        cs.skip_spaces()?;
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

    /// [Attribute](https://www.w3.org/TR/xml/#NT-Attribute)
    fn tokenize_attribute(cs: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        // spaces have already been skipped
        let name_range = Self::consume_name(cs)?;
        Self::expect_eq(cs)?;
        let used_quote = Self::consume_quote(cs)?;
        let value_range = Self::consume_character_data_until(cs, char::from(used_quote))?;
        cs.advance_n(1)?;
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
        let start_pos = cs.pos();
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
        Ok(Comment(cs.slice(start_pos..cs.pos())))
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
        let start_pos = cs.pos();
        let c = cs.next_xml_char()?;
        if !c.is_xml_name_start_char() {
            return Err(IllegalToken {
                range: cs.error_slice(start_pos..cs.pos()),
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
        Ok(cs.slice(start_pos..cs.pos()))
    }

    /// Consumes CharData until a specified char is found.
    /// By the standard, CharData cannot contain the literal & or < in addition to
    /// the CDATA section-close delimiter "]]>".
    /// However, the literal & can still be used to escape characters or define character references.
    ///
    /// CharData ::= \[^<&\]* - (\[^<&\]* ']]>' \[^<&\]*)
    /// [https://www.w3.org/TR/xml/#syntax]
    fn consume_character_data_until(cs: &mut CharIter<'a>, delimiter: char) -> Result<TextRange<'a>, XmlError> {
        let start_pos = cs.pos();
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
        Ok(cs.slice(start_pos..cs.pos()))
    }


    /// Consume any XML char until a specified byte slice is found
    fn consume_xml_chars_until(cs: &mut CharIter<'a>, delimiter: &[u8]) -> Result<TextRange<'a>, XmlError> {
        let start_pos = cs.pos();
        while !cs.test(delimiter) {
            cs.next_xml_char()?; // checks for valid XML char
        }
        Ok(cs.slice(start_pos..cs.pos()))
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
        let start_pos = cs.pos();
        cs.expect_byte(b'&')?;
        if cs.test(b"#x") {
            cs.skip_over(b"#x");

            // unicode char reference
            let char_hex_range = Self::consume_xml_chars_until(cs, b";")?;

            // decode character reference
            match util::decode_hex(char_hex_range.slice) {
                Some(_) => (),
                None => return Err(UnknownReference { range: cs.error_slice(start_pos..char_hex_range.end + 1) })
            };
        } else if cs.test(b"#") {
            cs.skip_over(b"#");

            // unicode char reference
            let code_point_range = Self::consume_xml_chars_until(cs, b";")?;
            let err = Err(UnknownReference { range: cs.error_slice(start_pos..code_point_range.end + 1) });
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
                _ => return Err(UnknownReference { range: cs.error_slice(start_pos..short_range.end + 1) })
            }
        }
        cs.skip_over(b";");
        println!("{:?}", &cs.text[start_pos..cs.pos()]);
        Ok(cs.slice(start_pos..cs.pos()))
    }

    /// [\[25\] Eq](https://www.w3.org/TR/xml/#NT-Eq)
    fn expect_eq(cs: &mut CharIter<'a>) -> Result<(), XmlError> {
        cs.skip_spaces();
        cs.expect_byte(b'=')?;
        cs.skip_spaces()?;
        Ok(())
    }

    /// ' or "
    fn consume_quote(cs: &mut CharIter<'a>) -> Result<u8, XmlError> {
        let quote = cs.next_byte()?;
        if !quote.is_xml_quote() {
            return Err(IllegalToken {
                range: cs.error_slice(cs.pos() - 1..cs.pos()),
                expected: Some("Either \" or '".to_string()),
            });
        }
        Ok(quote)
    }
}