use std::borrow::BorrowMut;
use std::iter::Peekable;
use std::slice::Iter;
use std::time::Instant;

use crate::{CharStream, DFA};
use crate::charstream::TextRange;
use crate::parse::XmlNode::*;
use crate::token::XmlToken::*;
use crate::token::XmlToken;
use crate::tokenstream::TokenStream;
use crate::xmlerror::{PositionalError, XmlTokenizeError};
use crate::xmlerror::XmlTokenizeError::UnknownToken;

#[derive(Debug)]
pub enum XmlNode<'a> {
    TextNode(&'a str),
    CommentNode(&'a str),
    ElementNode { name: &'a str, children: Vec<XmlNode<'a>> },
    AttributeNode { name: &'a str, value: &'a str },
    CdataSectionNode(&'a str),
    ProcessingInstructionNode(&'a str, Option<&'a str>),
}


#[inline]
fn slice<'a>(xml: &'a str, range: &TextRange) -> &'a str {
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

    fn unknown_token(xml: &'a str, token: &XmlToken) {
        panic!("{:?}", PositionalError::make_pos_error(xml,  token.encompassing_range().0,UnknownToken { token }));
    }

    fn parse_element(&mut self, xml: &'a str) -> XmlNode<'a> {
        let mut is_empty_element_tag = false;
        // TODO remove copy here, but there is a chance it is required
        let mut name_range = &match self.ts.next() {
            StartTag(name_range) => name_range.to_owned(),
            EmptyElementTag(name_range) => {
                is_empty_element_tag = true;
                name_range.to_owned()
            }
            token => {
                Self::unknown_token(xml, token);
                (0,0)
            }
        };
        let mut children = vec![];
        while let Attribute { name_range, value_range } = self.ts.peek() {
            let name = slice(xml, name_range);
            let value = slice(xml, value_range);
            children.push(AttributeNode { name, value });
            self.ts.next();
        }

        if !is_empty_element_tag {
            children.append(self.parse_element_content(xml).as_mut());
            // TODO verify end tag
            let end_tag = self.ts.next();
        }
        ElementNode { name: slice(xml, name_range), children }
    }

    /// content ::= CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    fn parse_element_content(&mut self, xml: &'a str) -> Vec<XmlNode<'a>> {
        let mut content = vec![];
        loop {
            match self.ts.peek() {
                EndTag(_) => break,
                EmptyElementTag(_) | StartTag(_) => {
                    content.push(self.parse_element(xml));
                }
                token => {
                    content.push(match token {
                        Text(value_range) => TextNode(slice(xml, value_range)),
                        Comment(value_range) => CommentNode(slice(xml, value_range)),
                        CdataSection(value_range) => CdataSectionNode(slice(xml, value_range)),
                        ProcessingInstruction { target_range, opt_value_range } =>
                            ProcessingInstructionNode(slice(xml, &target_range), opt_value_range.map(|ovr| slice(xml, &ovr))),
                        _ =>  {
                            Self::unknown_token(xml, token);
                            TextNode("sd")
                        }
                    });
                    self.ts.next();
                }
            }
        }
        content
    }
}