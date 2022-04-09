use std::collections::VecDeque;
use std::iter::FromIterator;
use std::str::Chars;
use std::thread::current;
use std::time::Instant;

use xmlparser::Stream;

use crate::CharStream;
use crate::charstream::TextRange;
use crate::token::XmlToken;
use crate::token::XmlToken::*;

pub struct DFA<'a> {
    pub(crate) cs: CharStream<'a>,
}


impl<'a> DFA<'a> {
    pub fn tokenize(&'a mut self) -> Vec<XmlToken> {
        let tokens = self.tokenize_markup();
        return tokens;
    }

    /// Aka element content
    /// content	:= CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    #[inline]
    pub fn tokenize_markup(&'a mut self) -> Vec<XmlToken> {
        let cs = &mut self.cs;
        let mut tokens = vec![];
        while cs.has_next() {
            let text_range = cs.consume_character_data_until('<');
            if !cs.range_is_empty(text_range) {
                tokens.push(Text(text_range));
            }
            if cs.upcoming("</") {
                tokens.push(Self::tokenize_end_tag(cs));
            } else if cs.upcoming("<!--") {
                tokens.push(Self::tokenize_comment(cs));
            } else if cs.upcoming("<![CDATA[") {
                tokens.push(Self::tokenize_cdata_section(cs));
            } else if cs.upcoming("<?") {
                tokens.push(Self::tokenize_processing_instruction(cs))
            } else {
                tokens.append(Self::tokenize_start_tag(cs).as_mut());
            }
        }
        tokens
    }


    /// STag ::= '<' Name (S Attribute)* S? '>'///
    /// EmptyElemTag ::= '<' Name (S Attribute)* S? '/>
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    #[inline]
    pub fn tokenize_start_tag(cs: &mut CharStream<'a>) -> Vec<XmlToken> {
        let mut tokens = vec![];

        cs.expect("<");
        let name_range = cs.consume_name();
        cs.skip_spaces();

        while !cs.upcoming("/>") && !cs.upcoming(">") {
            tokens.push(Self::tokenize_attribute(cs));
        }

        // Empty Element Tag
        let is_empty_element_tag = cs.upcoming("/>");
        if is_empty_element_tag {
            cs.expect("/>");
        } else {
            cs.expect(">")
        }

        tokens.insert(0, if is_empty_element_tag { EmptyElementTag(name_range) } else { StartTag(name_range) });
        tokens
    }

    /// ETag ::= '</' Name S? '>'
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    #[inline]
    pub fn tokenize_end_tag(cs: &mut CharStream<'a>) -> XmlToken {
        cs.expect("</");
        let name_range = cs.consume_name();
        cs.skip_spaces();
        cs.expect(">");
        return EndTag(name_range);
    }

    /// Attribute ::= Name Eq AttValue
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    #[inline]
    pub fn tokenize_attribute(cs: &mut CharStream<'a>) -> XmlToken {
        // spaces have already been skipped
        let name_range = cs.consume_name();
        cs.expect("=");
        let used_quote = cs.next_char();
        let value_range = cs.consume_character_data_until(used_quote);
        cs.skip_n(1);
        Attribute { name_range, value_range }
    }

    /// CDSect ::= CDStart CData CDEnd
    /// CDStart	::= '<![CDATA['
    /// CData ::= (Char* - (Char* ']]>' Char*))
    /// CDEnd ::= ']]>'
    /// [https://www.w3.org/TR/xml/#sec-cdata-sect]
    #[inline]
    pub fn tokenize_cdata_section(cs: &mut CharStream<'a>) -> XmlToken {
        cs.expect("<![CDATA[");
        let value_range = cs.consume_chars_until("]]>");
        cs.expect("]]>");
        CdataSection(value_range)
    }

    /// Comment ::= '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'
    /// [https://www.w3.org/TR/xml/#sec-comments]
    #[inline]
    pub fn tokenize_comment(cs: &mut CharStream<'a>) -> XmlToken {
        cs.expect("<!--");
        let value_range = cs.consume_comment();
        cs.expect("-->");
        Comment(value_range)
    }

    /// PI ::= '<?' PITarget (S (Char* - (Char* '?>' Char*)))? '?>'
    /// PITarget ::= Name - (('X' | 'x') ('M' | 'm') ('L' | 'l'))
    /// [https://www.w3.org/TR/xml/#sec-pi]
    pub fn tokenize_processing_instruction(cs: &mut CharStream<'a>) -> XmlToken {
        cs.expect("<?");

        let target_range = cs.consume_name();

        cs.skip_spaces();
        // TODO handle XML in processing instruction

        let mut opt_value_range = None;
        if !cs.upcoming("?>") {
            opt_value_range = Some(cs.consume_chars_until("?>"));
        }
        cs.expect("?>");
        ProcessingInstruction { target_range, opt_value_range }
    }
}