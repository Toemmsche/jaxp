use crate::error::*;
use crate::error::XmlError::UnexpectedXmlToken;
use crate::node::XmlNode;
use crate::node::XmlNode::*;
use crate::token::XmlToken::*;
use crate::tokenize::XmlTokenizer;
use crate::tokenstream::TokenStream;

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
        let ts = &mut TokenStream::from(tokens);

        // 10 is a reasonable max depth
        let mut depth_stack = Vec::with_capacity(20);
        // shadow document root
        depth_stack.push(Vec::with_capacity(1));

        while ts.has_next() {
            let active_child_list = depth_stack.last_mut().unwrap();
            match ts.next() {
                EndTag(name_range) => {
                    //TODO verify name equality
                    let tag_name = name_range.slice;
                    // Currently active child list belongs to this element node
                    let node = ElementNode { name: tag_name, children: depth_stack.pop().unwrap() };
                    // Add element node to parent element
                    depth_stack.last_mut().unwrap().push(node);
                }
                StartTag(_) => {
                    // TODO remember start tag
                    // let tag_name = name_range.slice;
                    // Change active child list
                    let child_list = Vec::with_capacity(5);
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
                    // TODO position of unexpected token
                    return Err(UnexpectedXmlToken { pos: XmlErrorPos { row: 0, col: 0 } });
                }
            }
        }
        Ok(depth_stack.pop().unwrap().pop().unwrap())
    }
}