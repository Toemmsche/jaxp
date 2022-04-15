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
use crate::xmlerror::XmlError::UnexpectedXmlToken;

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
        self.ts = TokenStream::from(tokens);
        self.parse_tree(xml)
    }

    #[inline]
    fn parse_tree(&mut self, xml: &'a str) -> Result<XmlNode<'a>, XmlError> {
        // 10 is a reasonable max depth
        let mut depth_stack = Vec::with_capacity(20);
        // shadow document root
        depth_stack.push(Vec::with_capacity(1));


        while self.ts.has_next() {
            match self.ts.next() {
                EndTag(name_range) => {
                    //TODO verify name equality
                    let tag_name = slice(xml, name_range);
                    // Currently active child list belongs to this element node
                    let node = ElementNode { name: tag_name, children: depth_stack.pop().unwrap() };
                    // Add element node to parent element
                    depth_stack.last_mut().unwrap().push(node);
                }
                EmptyElementTag(name_range) => {
                    let tag_name = slice(xml, name_range);
                    let node = ElementNode { name: tag_name, children: self.parse_attributes(xml)? };
                    // Add element node to parent element
                    depth_stack.last_mut().unwrap().push(node);
                }
                StartTag(_) => {
                    // Change active child list
                    depth_stack.push(self.parse_attributes(xml)?);
                }
                token => {
                    /// content ::= CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*
                    /// [https://www.w3.org/TR/xml/#sec-starttags]
                    depth_stack.last_mut().unwrap().push(match token {
                        Text(value_range) => TextNode(slice(xml, value_range)),
                        Comment(value_range) => CommentNode(slice(xml, value_range)),
                        CdataSection(value_range) => CdataSectionNode(slice(xml, value_range)),
                        ProcessingInstruction { target_range, opt_value_range } =>
                            ProcessingInstructionNode(slice(xml, &target_range), opt_value_range.map(|ovr| slice(xml, &ovr))),
                        unexpected_token => {
                            return Err(UnexpectedXmlToken { input: xml.to_string(), token: unexpected_token.clone() });
                        }
                    });
                }
            }
        }
        Ok(depth_stack.pop().unwrap().pop().unwrap())
    }

    fn parse_attributes(&mut self, xml: &'a str) -> Result<Vec<XmlNode<'a>>, XmlError> {
        let mut attributes = Vec::with_capacity(3);
        while let Attribute { name_range, value_range } = self.ts.peek() {
            let name = slice(xml, name_range);
            let value = slice(xml, value_range);
            attributes.push(AttributeNode { name, value });
            self.ts.next();
        }
        Ok(attributes)
    }
}