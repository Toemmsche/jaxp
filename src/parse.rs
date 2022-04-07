use std::borrow::BorrowMut;
use std::iter::Peekable;
use std::slice::Iter;
use std::time::Instant;

use crate::{CharStream, DFA};
use crate::charstream::TextRange;
use crate::dfa::{XmlToken, XmlTokenType};
use crate::dfa::XmlTokenType::{AttributeKey, EmptyElementTag, EndTag, StartTag};
use crate::parse::XmlNodeType::{Attribute, CdataSection, Comment, Element, Text};

#[derive(Debug)]
pub enum XmlNodeType {
    Text,
    Comment,
    Element,
    Attribute,
    CdataSection,
    ProcessingInstruction,
}

#[derive(Debug)]
pub struct XmlNode<'a> {
    pub(crate) node_type: XmlNodeType,
    pub(crate) key: Option<&'a str>,
    pub(crate) value: Option<&'a str>,
    pub(crate) children: Option<Vec<XmlNode<'a>>>,
}

pub struct XmlParser {}


impl<'a> XmlNode<'a> {
    pub fn create_text(xml: &'a str, content: TextRange) -> XmlNode {
        XmlNode {
            node_type:
            Text,
            key: None,
            value: Some(&xml[content.0..content.1]),
            children: None,
        }
    }

    pub fn create_element(xml: &'a str, name: TextRange, children: Vec<XmlNode<'a>>) -> XmlNode<'a> {
        XmlNode {
            node_type: Element,
            key: Some(&xml[name.0..name.1]),
            value: None,
            children: Some(children),
        }
    }

    pub fn create_attribute(xml: &'a str, key: TextRange, value: TextRange) -> XmlNode<'a> {
        XmlNode {
            node_type: Attribute,
            key: Some(&xml[key.0..key.1]),
            value: Some(&xml[value.0..value.1]),
            children: None,
        }
    }
    pub fn create_cdata_section(xml: &'a str, content: TextRange) -> XmlNode<'a> {
        XmlNode {
            node_type: CdataSection,
            key: None,
            value: Some(&xml[content.0..content.1]),
            children: None,
        }
    }
    pub fn create_comment(xml: &'a str, content: TextRange) -> XmlNode<'a> {
        XmlNode {
            node_type: Comment,
            key: None,
            value: Some(&xml[content.0..content.1]),
            children: None,
        }
    }
}

impl<'a> XmlParser {
    #[inline]
    pub fn parse(&self, xml: &'a str) -> XmlNode<'a> {
        //let now = Instant::now();
        // tokenize
        let tokens = DFA {
            cs: CharStream { text: xml, pos: 0 }
        }.tokenize(xml);
        //println!("Tokenize took: {:.2?}", now.elapsed());
        //println!("{:?}", tokens.len());

        self.parse_document(xml, &tokens)
    }

    #[inline]
    fn parse_document(&self, xml: &'a str, tokens: &Vec<XmlToken>) -> XmlNode<'a> {
        // delegate for now
        self.parse_element(xml, tokens, 0).1
    }

    #[inline]
    fn parse_element(&self, xml: &'a str, tokens: &Vec<XmlToken>, mut pos: usize) -> (usize, XmlNode<'a>) {
        let start_tag = &tokens[pos];
        pos += 1;
        let mut children = vec![];
        while tokens[pos].token_type == AttributeKey {
            let attr_key = tokens[pos].content;
            pos += 1;
            let attr_value = tokens[pos].content;
            pos += 1;
            children.push(XmlNode::create_attribute(xml, attr_key, attr_value));
        }
        if start_tag.token_type != EmptyElementTag {
            let (mut new_pos, mut nodes) = self.parse_element_content(xml, tokens, pos);
            pos = new_pos;
            children.append(&mut nodes);
            let end_tag = &tokens[pos];
            pos += 1;
        }
        (pos, XmlNode::create_element(xml, start_tag.content, children))
    }

    /// content ::= CharData? ((element | Reference | CDSect | PI | Comment) CharData?)*
    /// [https://www.w3.org/TR/xml/#sec-starttags]
    #[inline]
    fn parse_element_content(&self, xml: &'a str, tokens: &Vec<XmlToken>, mut pos: usize) -> (usize, Vec<XmlNode<'a>>) {
        let mut content = vec![];
        while pos < tokens.len() {
            let token = &tokens[pos];
            match token.token_type {
                EndTag => break,
                EmptyElementTag | StartTag => {
                    let (new_pos, node) = self.parse_element(xml, tokens, pos);
                    pos = new_pos;
                    content.push(node);
                }
                _ => {
                    content.push(match token.token_type {
                        XmlTokenType::Text => XmlNode::create_text(xml, token.content),
                        XmlTokenType::Comment => XmlNode::create_cdata_section(xml, token.content),
                        XmlTokenType::CdataSection => XmlNode::create_comment(xml, token.content),
                        _ => panic!("Unknown token typ {:?}", token.token_type)
                    });
                    pos += 1;
                }
            }
        }
        (pos, content)
    }
}