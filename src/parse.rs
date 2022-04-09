use std::borrow::BorrowMut;
use std::iter::Peekable;
use std::slice::Iter;
use std::time::Instant;

use crate::{CharStream, DFA};
use crate::charstream::TextRange;
use crate::dfa::{XmlToken, XmlTokenType};
use crate::dfa::XmlTokenType::{AttributeKey, EmptyElementTag, EndTag, PIData, PITarget, StartTag};
use crate::parse::XmlNode::*;
use crate::tokenstream::TokenStream;

#[derive(Debug)]
pub enum XmlNode<'a> {
    Text(&'a str),
    Comment(&'a str),
    Element {tag_name: &'a str, children: Vec<XmlNode<'a>>},
    Attribute(&'a str, &'a str),
    CdataSection(&'a str),
    ProcessingInstruction(&'a str, Option<&'a str>),
}


#[inline]
fn slice(xml: &str, range: TextRange) -> &str {
    &xml[range.0..range.1]
}

pub struct XmlParser {
    pub(crate) ts: TokenStream,
}


impl<'a> XmlParser {


    #[inline]
    pub fn parse(&mut self, xml: &'a str) -> XmlNode<'a> {
        //let now = Instant::now();
        // tokenize
        let tokens = DFA {
            cs: CharStream { text: xml, pos: 0 }
        }.tokenize();
        self.ts = TokenStream { pos: 0, tokens };

        self.parse_document(xml)
    }

    #[inline]
    fn parse_document(&mut self, xml: &'a str) -> XmlNode<'a> {
        // delegate for now
        self.parse_element(xml)
    }

    fn parse_element(&mut self, xml: &'a str) -> XmlNode<'a> {
        let start_tag_type = self.ts.peek().token_type;
        let start_tag_range = self.ts.next().content;
        let mut children = vec![];
        while self.ts.peek().token_type == AttributeKey {
            let attr_key = slice(xml, self.ts.next().content);
            let attr_value = slice(xml,  self.ts.next().content);
            children.push(Attribute(attr_key, attr_value));
        }
        if start_tag_type != EmptyElementTag {
            let mut nodes = self.parse_element_content(xml);
            children.append(&mut nodes);
            let end_tag = self.ts.next();
        }
        Element {tag_name: slice(xml, start_tag_range), children}
    }

    /// content ::= CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    fn parse_element_content(&mut self, xml: &'a str) -> Vec<XmlNode<'a>> {
        let mut content = vec![];
        loop {
            let token = self.ts.peek();
            let range = token.content;
            match token.token_type {
                XmlTokenType::EndTag => break,
                XmlTokenType::EmptyElementTag | XmlTokenType::StartTag => {
                    content.push(self.parse_element(xml));
                }

                _ => {
                    content.push(match token.token_type {
                        XmlTokenType::Text => Text(slice(xml, range)),
                        XmlTokenType::Comment => CdataSection(slice(xml, range)),
                        XmlTokenType::CdataSection => Comment(slice(xml, range)),
                        XmlTokenType::PITarget => {
                            if self.ts.peek_n(1).token_type == XmlTokenType::PIData {
                                // Advance iterator an additional time
                                ProcessingInstruction(slice(xml, self.ts.next().content),Some(slice(xml,self.ts.peek().content)))
                            } else {
                                ProcessingInstruction(slice(xml, range), None)
                            }
                        }
                        _ => panic!("Unknown token typ {:?}", token.token_type)
                    });
                    self.ts.next();
                }
            }
        }
        content
    }
}