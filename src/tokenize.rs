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
        let mut ci = CharIter { pos: 0, text: xml };

        return Self::tokenize_document(&mut ci);
    }

    /// [\[1\] document](https://www.w3.org/TR/xml/#NT-document)
    fn tokenize_document(ci: &mut CharIter<'a>) -> Result<Vec<XmlToken<'a>>, XmlError> {
        let mut tokens = Self::tokenize_prolog(ci)?;
        tokens.append(&mut Self::tokenize_content(ci)?);
        return Ok(tokens);
    }

    /// [\[22\] prolog](https://www.w3.org/TR/xml/#NT-prolog)
    fn tokenize_prolog(ci: &mut CharIter<'a>) -> Result<Vec<XmlToken<'a>>, XmlError> {
        let mut tokens = vec![];
        if ci.test(b"<?xml") {
            tokens.push(Self::tokenize_xml_declaration(ci)?);
        }
        while ci.peek_byte()?.is_xml_whitespace() || ci.test(b"<!--") || ci.test(b"<?") {
            // TODO lift space here for performance
            match Self::tokenize_misc(ci)? {
                Some(token) => tokens.push(token),
                None => ()
            }
        }
        if ci.test(b"<!DOCTYPE") {
            tokens.append(&mut Self::tokenize_doctype_declaration(ci)?);

            while ci.peek_byte()?.is_xml_whitespace() || ci.test(b"<!--") || ci.test(b"<?") {
                // TODO lift space here for performance
                match Self::tokenize_misc(ci)? {
                    Some(token) => tokens.push(token),
                    None => ()
                }
            }
        }
        Ok(tokens)
    }

    /// [\[27\] Misc](https://www.w3.org/TR/xml/#NT-Misc)
    fn tokenize_misc(ci: &mut CharIter<'a>) -> Result<Option<XmlToken<'a>>, XmlError> {
        return if ci.peek_byte()?.is_xml_whitespace() {
            ci.advance_n(1)?;
            Ok(None)
        } else if ci.test(b"<!--") {
            Ok(Some(Self::tokenize_comment(ci)?))
        } else if ci.test(b"<?") {
            Ok(Some(Self::tokenize_processing_instruction(ci)?))
        } else {
            Err(IllegalToken {
                pos: ci.error_pos(),
                expected: Some("Space or Start of Comment or Processing Instruction".to_string()),
            })
        };
    }

    /// [\[28b\] intSubset](https://www.w3.org/TR/xml/#NT-intSubset)
    fn tokenize_internal_subset(ci: &mut CharIter<'a>) -> Result<Vec<XmlToken<'a>>, XmlError> {
        let mut tokens = vec![];
        while !ci.test_byte(b']') {
            // [\[28a\] DeclSep](https://www.w3.org/TR/xml/#NT-DeclSep)
            ci.skip_spaces();
            if ci.test_byte(b'%') {
                tokens.push(ParameterEntityReference(Self::consume_parameter_entity_reference(ci)?));
            } else {
                // TODO test for markup declarations
            }
        }
        Ok(tokens)
    }

    /// [\[69\] PEReference](https://www.w3.org/TR/xml/#NT-PEReference)
    fn consume_parameter_entity_reference(ci: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        ci.expect_byte(b'%')?;
        let name_range = Self::consume_name(ci)?;
        ci.expect_byte(b';')?;
        Ok(name_range)
    }

    /// [\[28\] doctypedecl](https://www.w3.org/TR/xml/#NT-doctypedecl)
    fn tokenize_doctype_declaration(ci: &mut CharIter<'a>) -> Result<Vec<XmlToken<'a>>, XmlError> {
        let mut tokens = vec![];
        ci.expect_bytes(b"<!DOCTYPE")?;
        ci.expect_spaces()?;
        let name_range = Self::consume_name(ci)?;
        let mut opt_system_entity_range = None;
        let mut opt_public_entity_range = None;
        // externalID ?
        if ci.test_after_spaces(b"PUBLIC") || ci.test_after_spaces(b"SYSTEM") {
            ci.expect_spaces()?;
            (opt_system_entity_range, opt_public_entity_range) = Self::consume_external_id(ci)?;
        }
        tokens.push(DocTypeDeclaration {
            name_range,
            opt_system_entity_range,
            opt_public_entity_range,
        });
        ci.skip_spaces();
        if ci.test_byte(b'[') {
            ci.advance_n(1)?;
            tokens.append(&mut Self::tokenize_internal_subset(ci)?);
            ci.expect_byte(b']')?;
        }
        ci.skip_spaces();
        ci.expect_byte(b'>')?;
        Ok(tokens)
    }

    /// [\[75\] ExternalID](https://www.w3.org/TR/xml/#NT-ExternalID)
    fn consume_external_id(ci: &mut CharIter<'a>) -> Result<(Option<TextRange<'a>>, Option<TextRange<'a>>), XmlError> {
        let system_start_delimiter = b"SYSTEM";
        let public_start_delimiter = b"PUBLIC";
        return if ci.test(system_start_delimiter) {
            ci.skip_over(system_start_delimiter)?;
            ci.expect_spaces()?;
            let system_literal_range = Self::consume_system_literal(ci)?;
            Ok((Some(system_literal_range), None))
        } else if ci.test(public_start_delimiter) {
            ci.skip_over(public_start_delimiter)?;
            ci.expect_spaces()?;
            let pubid_literal_range = Self::consume_pubid_literal(ci)?;
            ci.expect_spaces()?;
            let system_literal_range = Self::consume_system_literal(ci)?;
            Ok((Some(system_literal_range), Some(pubid_literal_range)))
        } else {
            Err(IllegalToken {
                pos: ci.error_pos(),
                expected: Some("'SYSTEM' or 'PUBLIC'".to_string()),
            })
        };
    }

    /// [\[11\] SystemLiteral](https://www.w3.org/TR/xml/#NT-SystemLiteral)
    fn consume_system_literal(ci: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        let used_quote = Self::consume_quote(ci)?;
        let literal_range = Self::consume_xml_chars_until(ci, &[used_quote])?;
        ci.expect_byte(used_quote)?;
        Ok(literal_range)
    }

    /// [\[12\] PubidLiteral](https://www.w3.org/TR/xml/#NT-PubidLiteral)
    fn consume_pubid_literal(ci: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        let used_quote = Self::consume_quote(ci)?;
        let start_pos = ci.pos();
        while ci.peek_byte()?.is_xml_pubid_char() && ci.peek_byte()? != used_quote {
            ci.advance_n(1)?;
        }
        let literal_range = ci.slice(start_pos..ci.pos());
        ci.expect_byte(used_quote)?;
        Ok(literal_range)
    }


    /// [\[23\] XMLDecl](https://www.w3.org/TR/xml/#NT-XMLDecl)
    fn tokenize_xml_declaration(ci: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        ci.skip_over(b"<?xml")?;
        let xml_decl_end_delim = b"?>";
        let version_info_range = Self::consume_version_info(ci)?;
        let mut encoding_declaration_range = None;
        let mut standalone_document_declaration_range = None;
        if ci.test_after_spaces(b"encoding") {
            encoding_declaration_range = Some(Self::consume_encoding_declaration(ci)?);
        }
        if ci.test_after_spaces(b"standalone") {
            standalone_document_declaration_range = Some(Self::consume_standalone_document_declaration(ci)?);
        }
        ci.skip_spaces();
        ci.expect_bytes(xml_decl_end_delim)?;
        Ok(XmlDeclaration {
            version_range: version_info_range,
            opt_encoding_range: encoding_declaration_range,
            opt_standalone_range: standalone_document_declaration_range,
        })
    }


    /// [\[32\] SDDecl](https://www.w3.org/TR/xml/#NT-SDDecl)
    fn consume_standalone_document_declaration(ci: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        ci.expect_spaces()?;
        ci.expect_bytes(b"standalone")?;
        Self::expect_eq(ci)?;
        let used_quote = Self::consume_quote(ci)?;

        let start_pos = ci.pos();
        if ci.test(b"yes") {
            ci.skip_over(b"yes")?;
        } else if ci.test(b"no") {
            ci.skip_over(b"no")?;
        } else {
            return Err(
                IllegalToken {
                    pos: ci.error_pos(),
                    expected: Some("yes or no".to_string()),
                }
            );
        }
        let end_pos = ci.pos();
        ci.expect_byte(used_quote)?;
        return Ok(ci.slice(start_pos..end_pos));
    }


    /// [\[80\] EncodingDecl](https://www.w3.org/TR/xml/#NT-EncodingDecl)
    fn consume_encoding_declaration(ci: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        ci.expect_spaces()?;
        ci.expect_bytes(b"encoding")?;
        Self::expect_eq(ci)?;
        let used_quote = Self::consume_quote(ci)?;

        let range = Self::consume_encoding_name(ci)?;
        ci.expect_byte(used_quote)?;
        return Ok(range);
    }

    /// [\[81\] EncName](https://www.w3.org/TR/xml/#NT-VersionNum)
    fn consume_encoding_name(ci: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        let start_pos = ci.pos();
        /* Encoding name contains only Latin characters */
        let byte = ci.next_byte()?;
        if !byte.is_ascii_alphabetic() {
            return Err(IllegalToken {
                pos: ci.error_pos(),
                expected: Some("Any latin letter".to_string()),
            });
        }
        // maybe move this to xmlchar
        while ci.peek_byte()?.is_ascii_alphanumeric() || match ci.peek_byte()? {
            b'.' | b'_' | b'-' => true,
            _ => false
        } {
            ci.advance_n(1)?;
        }
        Ok(ci.slice(start_pos..ci.pos()))
    }


    /// [\[24\] VersionInfo](https://www.w3.org/TR/xml/#NT-VersionInfo)
    fn consume_version_info(ci: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        ci.expect_spaces()?;
        ci.expect_bytes(b"version")?;
        Self::expect_eq(ci)?;
        let used_quote = Self::consume_quote(ci)?;

        let range = Self::consume_version_num(ci)?;
        ci.expect_byte(used_quote)?;
        return Ok(range);
    }

    /// [\[26\] VersionNUm](https://www.w3.org/TR/xml/#NT-VersionNum)
    fn consume_version_num(ci: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        let start_pos = ci.pos();
        ci.expect_bytes(b"1.")?;
        // TODO: remove redundant xml_char check
        while ci.peek_xml_char()?.is_ascii_digit() {
            ci.next_xml_char()?;
        }
        Ok(ci.slice(start_pos..ci.pos()))
    }

    /// [\[43\] content](https://www.w3.org/TR/xml/#NT-content)
    fn tokenize_content(ci: &mut CharIter<'a>) -> Result<Vec<XmlToken<'a>>, XmlError> {
        // average token length of ~20 bytes
        let mut tokens = Vec::with_capacity(ci.text.len() / 20);
        while ci.has_next() {
            let text_range = Self::consume_character_data_until(ci, '<')?;
            if !text_range.is_empty() {
                tokens.push(Text(text_range));
            }
            if ci.test(b"</") {
                tokens.push(Self::tokenize_end_tag(ci)?);
            } else if ci.test(b"<!--") {
                tokens.push(Self::tokenize_comment(ci)?);
            } else if ci.test(b"<![CDATA[") {
                tokens.push(Self::tokenize_cdata_section(ci)?);
            } else if ci.test(b"<?") {
                tokens.push(Self::tokenize_processing_instruction(ci)?)
            } else {
                tokens.append(Self::tokenize_start_tag(ci)?.as_mut());
            }
        }
        Ok(tokens)
    }


    /// [\[40\] STag](https://www.w3.org/TR/xml/#NT-STag)
    fn tokenize_start_tag(ci: &mut CharIter<'a>) -> Result<Vec<XmlToken<'a>>, XmlError> {
        let mut tokens = vec![];

        //tag start has already been identified
        ci.skip_over(b"<")?;
        let name_range = Self::consume_name(ci)?;

        while !ci.test_after_spaces(b"/>") && !ci.test_after_spaces(b">") {
            ci.expect_spaces()?;
            tokens.push(Self::tokenize_attribute(ci)?);
        }

        ci.skip_spaces();
        // Empty Element Tag
        let is_empty_element_tag = ci.test(b"/>");
        if is_empty_element_tag {
            ci.expect_bytes(b"/>")?;
        } else {
            ci.expect_byte(b'>')?;
        }

        tokens.insert(0, StartTag(name_range));
        if is_empty_element_tag {
            // Create artificial end tag
            tokens.push(EndTag(name_range));
        }
        Ok(tokens)
    }

    /// [\[42\] ETag](https://www.w3.org/TR/xml/#NT-ETag)
    fn tokenize_end_tag(ci: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        ci.skip_over(b"</")?;
        let name_range = Self::consume_name(ci)?;
        ci.skip_spaces();
        ci.expect_byte(b'>')?;
        Ok(EndTag(name_range))
    }

    /// [\[41\] Attribute](https://www.w3.org/TR/xml/#NT-Attribute)
    fn tokenize_attribute(ci: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        // spaces have already been skipped
        let name_range = Self::consume_name(ci)?;
        Self::expect_eq(ci)?;
        let used_quote = Self::consume_quote(ci)?;
        // TODO consider references in Attributes
        // [\[10\] AttValue](https://www.w3.org/TR/xml/#NT-AttValue)
        let value_range = Self::consume_character_data_until(ci, char::from(used_quote))?;
        ci.advance_n(1)?;
        Ok(Attribute { name_range, value_range })
    }

    /// [\[18\] CDSect](https://www.w3.org/TR/xml/#NT-CDSect)
    fn tokenize_cdata_section(ci: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        ci.skip_over(b"<![CDATA[")?;
        let value_range = Self::consume_xml_chars_until(ci, b"]]>")?;
        ci.skip_over(b"]]>")?;
        Ok(CdataSection(value_range))
    }

    /// [\[15\] Comment](https://www.w3.org/TR/xml/#NT-Comment)
    fn tokenize_comment(ci: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        ci.skip_over(b"<!--")?;
        let start_pos = ci.pos();
        loop {
            if ci.test(b"--") {
                if ci.test(b"-->") {
                    break;
                } else if ci.test(b"--->") {
                    // Last character cannot be a hyphen
                    return Err(IllegalToken {
                        pos: ci.error_pos(),
                        expected: Some("Not a hyphen as the last value character".to_string()),
                    });
                } else {
                    // Double hypen is not allowed inside comments
                    return Err(IllegalToken {
                        pos: ci.error_pos(),
                        expected: Some("Not a double hyphen inside comments".to_string()),
                    });
                }
            }
            ci.next_xml_char()?;
        }
        let value_range = ci.slice(start_pos..ci.pos());
        ci.skip_over(b"-->")?;
        Ok(Comment(value_range))
    }

    /// [\[16\] PI](https://www.w3.org/TR/xml/#NT-PI)
    fn tokenize_processing_instruction(ci: &mut CharIter<'a>) -> Result<XmlToken<'a>, XmlError> {
        ci.skip_over(b"<?")?;
        let target_range = Self::consume_name(ci)?;
        ci.skip_spaces();

        // TODO forbid literal "XML" in processing instruction
        let mut opt_value_range = None;
        if !ci.test(b"?>") {
            opt_value_range = Some(Self::consume_xml_chars_until(ci, b"?>")?);
        }

        ci.skip_over(b"?>")?;
        Ok(ProcessingInstruction { target_range, opt_value_range })
    }

    /// [\[5\] Name](https://www.w3.org/TR/xml/#NT-Name)
    pub fn consume_name(ci: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        let start_pos = ci.pos();
        let c = ci.next_xml_char()?;
        if !c.is_xml_name_start_char() {
            return Err(IllegalToken {
                pos: ci.error_pos(),
                expected: Some("Any Name start char".to_string()),
            });
        }
        loop {
            let c = ci.peek_xml_char()?;
            if c.is_xml_name_char() {
                ci.advance_n(c.len_utf8())?;
            } else {
                break;
            }
        }
        Ok(ci.slice(start_pos..ci.pos()))
    }

    /// Consumes CharData until a specified char is found.
    /// By the standard, CharData cannot contain the literal & or < in addition to
    /// the CDATA section-close delimiter "]]>".
    /// However, the literal & can still be used to escape characters or define character references.
    ///
    /// CharData ::= \[^<&\]* - (\[^<&\]* ']]>' \[^<&\]*)
    /// [\[14\] CharData](https://www.w3.org/TR/xml/#NT-CharData)
    fn consume_character_data_until(ci: &mut CharIter<'a>, delimiter: char) -> Result<TextRange<'a>, XmlError> {
        let start_pos = ci.pos();
        let cdata_close_delimiter = b"]]>";
        loop {
            match ci.peek_xml_char()? {
                c if c == delimiter => break,
                ']' => if ci.test(cdata_close_delimiter) {
                    return Err(IllegalToken {
                        pos: ci.error_pos(),
                        expected: Some("Not the CDATA section-close delimiter".to_string()),
                    });
                },
                '&' => {
                    // TODO handle returned range
                    Self::consume_character_reference(ci)?;
                    continue;
                }
                '<' => {
                    return Err(IllegalToken {
                        pos: ci.error_pos(),
                        expected: Some("Not the less-than character".to_string()),
                    });
                }
                c => { ci.advance_n(c.len_utf8())?; }
            }
        }
        Ok(ci.slice(start_pos..ci.pos()))
    }


    /// Consume any XML char until a specified byte slice is found
    fn consume_xml_chars_until(ci: &mut CharIter<'a>, delimiter: &[u8]) -> Result<TextRange<'a>, XmlError> {
        let start_pos = ci.pos();
        while !ci.test(delimiter) {
            ci.next_xml_char()?; // checks for valid XML char
        }
        Ok(ci.slice(start_pos..ci.pos()))
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
    /// [\[66\] CharRef](https://www.w3.org/TR/xml/#NT-CharRef)
    fn consume_character_reference(ci: &mut CharIter<'a>) -> Result<TextRange<'a>, XmlError> {
        let start_pos = ci.pos();
        ci.expect_byte(b'&')?;
        if ci.test(b"#x") {
            ci.skip_over(b"#x")?;

            // unicode char reference
            let char_hex_range = Self::consume_xml_chars_until(ci, b";")?;

            // decode character reference
            match util::decode_hex(char_hex_range.slice) {
                Some(_) => (),
                None => return Err(UnknownReference {
                    pos: ci.error_pos()
                })
            };
        } else if ci.test(b"#") {
            ci.skip_over(b"#")?;

            // unicode char reference
            let code_point_range = Self::consume_xml_chars_until(ci, b";")?;
            let err = Err(UnknownReference {
                pos: ci.error_pos()
            });
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
            let short_range = Self::consume_xml_chars_until(ci, b";")?;
            match short_range.slice {
                "amp" | "lt" | "gt" | "apos" | "quot" => (), // all good
                _ => return Err(UnknownReference {
                    pos: ci.error_pos()
                })
            }
        }
        ci.skip_over(b";")?;
        Ok(ci.slice(start_pos..ci.pos()))
    }

    /// [\[25\] Eq](https://www.w3.org/TR/xml/#NT-Eq)
    fn expect_eq(ci: &mut CharIter<'a>) -> Result<(), XmlError> {
        ci.skip_spaces();
        ci.expect_byte(b'=')?;
        ci.skip_spaces();
        Ok(())
    }

    /// ' or "
    fn consume_quote(ci: &mut CharIter<'a>) -> Result<u8, XmlError> {
        let quote = ci.next_byte()?;
        if !quote.is_xml_quote() {
            return Err(IllegalToken {
                pos: ci.error_pos(),
                expected: Some("Either \" or '".to_string()),
            });
        }
        Ok(quote)
    }
}