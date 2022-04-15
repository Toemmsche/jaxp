use std::collections::VecDeque;
use std::iter::FromIterator;
use std::str::Chars;
use std::thread::current;
use std::time::Instant;

use xmlparser::Stream;

use crate::charstream::{CharStream, TextRange};
use crate::token::XmlRangeToken;
use crate::token::XmlRangeToken::*;
use crate::xmlerror::XmlError;

pub struct XmlTokenizer<'a> {
    pub(crate) cs: CharStream<'a>,
}

impl Default for XmlTokenizer<'_> {
    fn default() -> Self {
        XmlTokenizer {
            cs: CharStream::default()
        }
    }
}


impl<'a> XmlTokenizer<'a> {
    pub fn tokenize(&mut self, xml: &'a str) -> Result<Vec<XmlRangeToken>, XmlError> {
        self.cs = CharStream { pos: 0, text: xml };
        return self.tokenize_markup();
    }

    /// Aka element content
    /// content	:= CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    #[inline]
    fn tokenize_markup(&mut self) -> Result<Vec<XmlRangeToken>, XmlError> {
        let cs = &mut self.cs;
        // average token length of ~20 bytes
        let mut tokens = Vec::with_capacity(cs.text.len() / 20);
        while cs.has_next() {
            let text_range = cs.consume_character_data_until('<')?;
            if !cs.range_is_empty(text_range) {
                tokens.push(Text(text_range));
            }
            if cs.upcoming(b"</") {
                tokens.push(Self::tokenize_end_tag(cs)?);
            } else if cs.upcoming(b"<!--") {
                tokens.push(Self::tokenize_comment(cs)?);
            } else if cs.upcoming(b"<![CDATA[") {
                tokens.push(Self::tokenize_cdata_section(cs)?);
            } else if cs.upcoming(b"<?") {
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
    #[inline]
    fn tokenize_start_tag(cs: &mut CharStream<'a>) -> Result<Vec<XmlRangeToken>, XmlError> {
        let mut tokens = vec![];

        //tag start has already been identified
        cs.skip_over(b"<");
        let name_range = cs.consume_name()?;
        cs.skip_spaces()?;

        while !cs.upcoming(b"/>") && !cs.upcoming(b">") {
            tokens.push(Self::tokenize_attribute(cs)?);
        }

        // Empty Element Tag
        let is_empty_element_tag = cs.upcoming(b"/>");
        if is_empty_element_tag {
            cs.expect_bytes(b"/>")?;
        } else {
            cs.expect_byte(b'>')?;
        }

        tokens.insert(0, if is_empty_element_tag { EmptyElementTag(name_range) } else { StartTag(name_range) });
        Ok(tokens)
    }

    /// ETag ::= '</' Name S? '>'
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    #[inline]
    fn tokenize_end_tag(cs: &mut CharStream) -> Result<XmlRangeToken, XmlError> {
        cs.skip_over(b"</");
        let name_range = cs.consume_name()?;
        cs.skip_spaces()?;
        cs.expect_byte(b'>')?;
        Ok(EndTag(name_range))
    }

    /// Attribute ::= Name Eq AttValue
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    #[inline]
    fn tokenize_attribute(cs: &mut CharStream<'a>) -> Result<XmlRangeToken, XmlError> {
        // spaces have already been skipped
        let name_range = cs.consume_name()?;
        cs.expect_byte(b'=')?;
        let used_quote = cs.next_char()?;
        let value_range = cs.consume_character_data_until(used_quote)?;
        cs.advance_n(used_quote.len_utf8());
        Ok(Attribute { name_range, value_range })
    }

    /// CDSect ::= CDStart CData CDEnd
    /// CDStart	::= '<![CDATA['
    /// CData ::= (Char* - (Char* ']]>' Char*))
    /// CDEnd ::= ']]>'
    /// [https://www.w3.org/TR/xml/#sec-cdata-sect]
    #[inline]
    fn tokenize_cdata_section(cs: &mut CharStream) -> Result<XmlRangeToken, XmlError> {
        cs.skip_over(b"<![CDATA[");
        let value_range = cs.consume_chars_until(b"]]>")?;
        cs.skip_over(b"]]>");
        Ok(CdataSection(value_range))
    }

    /// Comment ::= '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'
    /// [https://www.w3.org/TR/xml/#sec-comments]
    #[inline]
    fn tokenize_comment(cs: &mut CharStream) -> Result<XmlRangeToken, XmlError> {
        cs.skip_over(b"<!--");
        let value_range = cs.consume_comment()?;
        cs.skip_over(b"-->");
        Ok(Comment(value_range))
    }

    /// PI ::= '<?' PITarget (S (Char* - (Char* '?>' Char*)))? '?>'
    /// PITarget ::= Name - (('X' | 'x') ('M' | 'm') ('L' | 'l'))
    /// [https://www.w3.org/TR/xml/#sec-pi]
    #[inline]
    fn tokenize_processing_instruction(cs: &mut CharStream) -> Result<XmlRangeToken, XmlError> {
        cs.skip_over(b"<?");
        let target_range = cs.consume_name()?;
        cs.skip_spaces()?;
        // TODO handle XML in processing instruction

        let mut opt_value_range = None;
        if !cs.upcoming(b"?>") {
            opt_value_range = Some(cs.consume_chars_until(b"?>")?);
        }

        cs.skip_over(b"?>");
        Ok(ProcessingInstruction { target_range, opt_value_range })
    }
}