use std::borrow::BorrowMut;
use std::iter::Peekable;
use std::slice::Iter;
use std::str::FromStr;
use std::time::Instant;

use crate::charstream::{CharStream, TextRange};
use crate::node::XmlNode;
use crate::node::XmlNode::*;
use crate::token::XmlRangeToken::*;
use crate::token::XmlRangeToken;
use crate::tokenize::XmlTokenizer;
use crate::tokenstream::TokenStream;
use crate::xmlerror::*;

#[inline]
fn slice<'a>(xml: &'a str, range: &TextRange) -> &'a str {
    &xml[range.0..range.1]
}

pub struct XmlParser {
    pub(crate) ts: TokenStream,
}

impl Default for XmlParser {
    fn default() -> Self {
        XmlParser {
            ts: TokenStream::default()
        }
    }
}

impl<'a> XmlParser {
    #[inline]
    pub fn parse(&mut self, xml: &'a str) -> Result<XmlNode<'a>, XmlError> {
        // tokenize
        let tokens = XmlTokenizer::default().tokenize(xml)?;
        self.ts = TokenStream { pos: 0, tokens };
        Ok(self.parse_document(xml))
    }

    #[inline]
    fn parse_document(&mut self, xml: &'a str) -> XmlNode<'a> {
        // delegate for now
        self.parse_element(xml)
    }

    fn parse_element(&mut self, xml: &'a str) -> XmlNode<'a> {
        let mut is_empty_element_tag = false;
        // TODO remove copy here, but there is a chance it is required
        let mut tag_name = match self.ts.next() {
            StartTag(name_range) => slice(xml, name_range),
            EmptyElementTag(name_range) => {
                is_empty_element_tag = true;
                slice(xml, name_range)
            }
            _ => {
                panic!("sdf");
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
        ElementNode { name: tag_name, children }
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
                        _ => {
                            TextNode("ERROR ERROR")
                        }
                    });
                    self.ts.next();
                }
            }
        }
        content
    }
}