use std::borrow::BorrowMut;
use std::iter::Peekable;
use std::slice::Iter;
use std::str::FromStr;
use std::time::Instant;

use crate::charstream::CharIter;
use crate::node::XmlNode;
use crate::node::XmlNode::*;
use crate::textrange::TextRange;
use crate::token::XmlToken::*;
use crate::token::XmlToken;
use crate::tokenize::XmlTokenizer;
use crate::tokenstream::TokenStream;
use crate::xmlerror::*;
use crate::xmlerror::XmlError::{InternalError, UnexpectedXmlToken};

pub struct XmlParser {}

impl Default for XmlParser {
    fn default() -> Self {
        XmlParser {}
    }
}

impl<'a> XmlParser {
    pub fn parse(&mut self, xml: &'a str) -> Result<XmlNode<'a>, XmlError> {
        // tokenize
        let tokens = XmlTokenizer::default().tokenize(xml)?;
        let mut ts = &mut TokenStream::from(tokens);

        // 10 is a reasonable max depth
        let mut depth_stack = Vec::with_capacity(20);
        // shadow document root
        depth_stack.push(Vec::with_capacity(1));

        while ts.has_next() {
            let mut active_child_list = depth_stack.last_mut().unwrap();
            match ts.next() {
                EndTag(name_range) => {
                    //TODO verify name equality
                    let tag_name = name_range.slice;
                    // Currently active child list belongs to this element node
                    let node = ElementNode { name: tag_name, children: depth_stack.pop().unwrap() };
                    // Add element node to parent element
                    depth_stack.last_mut().unwrap().push(node);
                }
                StartTag(name_range) => {
                    let tag_name = name_range.slice;
                    // Change active child list
                    let mut child_list = Vec::with_capacity(5);
                    depth_stack.push(child_list);
                }
                Attribute { name_range, value_range } => {
                    active_child_list.push(AttributeNode { name: name_range.slice, value: value_range.slice })
                }
                Text(value_range) =>
                    active_child_list.push(TextNode(value_range.slice)),
                Comment(value_range) =>
                    active_child_list.push(CommentNode(value_range.slice)),
                CdataSection(value_range) =>
                    active_child_list.push(CdataSectionNode(value_range.slice)),
                ProcessingInstruction { target_range, opt_value_range } =>
                    active_child_list.push(ProcessingInstructionNode(target_range.slice, opt_value_range.map(|ovr| ovr.slice))),
                unexpected_token => {
                    // TODO add unexpected token error
                    return Err(InternalError);
                }
            }
        }
        Ok(depth_stack.pop().unwrap().pop().unwrap())
    }
}