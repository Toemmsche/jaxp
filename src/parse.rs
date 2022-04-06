use std::time::Instant;
use crate::default_tokenizer;
use crate::dfa::XmlToken;
use crate::parse::XmlNodeType::{Attribute, Element, Text};
use crate::token::{XmlToken, XmlTokenType};
use crate::dfa::XmlTokenType::{AttributeKey, ClosingTag, EmptyElementTag, OpeningTag};

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
pub struct XmlNode {
    node_type: XmlNodeType,
    key: Option<String>,
    value: Option<String>,
    children: Option<Vec<XmlNode>>,
}

pub struct XmlParser {
    pos: usize,
    tokens: Vec<XmlToken>,
}

pub fn create_text(content: String) -> XmlNode {
    XmlNode { node_type: Text, key: None, value: Some(content), children: None }
}

pub fn create_element(name: String, children: Vec<XmlNode>) -> XmlNode {
    XmlNode {
        node_type: Element,
        key: Some(name),
        value: None,
        children: Some(children),
    }
}

pub fn default_parser() -> XmlParser {
    XmlParser { pos: 0, tokens: vec![] }
}

impl XmlParser {
    fn peek(&self) -> XmlToken {
        self.tokens[self.pos].to_owned()
    }

    fn next(&mut self) -> XmlToken {
        let token = self.peek();
        self.pos += 1;
        token
    }

    pub fn parse(&mut self, xml: String) -> XmlNode {
        let now = Instant::now();
        self.tokens = default_tokenizer().tokenize(xml);

        let elapsed = now.elapsed();
        println!("Elapsed for tokenize: {:.2?}", elapsed);

        self.pos = 0;
        self.parse_element()
    }

    fn parse_element(&mut self) -> XmlNode {
        let opening_tag = self.next();
        let mut children = vec![];
        if opening_tag.token_type != EmptyElementTag {
            while self.peek().token_type == AttributeKey {
                let attr_key = self.next().content.to_owned();
                let attr_value = self.next().content.to_owned();
                children.push(XmlNode {
                    node_type: Attribute,
                    key: Some(attr_key),
                    value: Some(attr_value),
                    children: None,
                })
            }
            children.append(&mut self.parse_element_content());
            let closing_tag = self.next();
            if closing_tag.content != opening_tag.content {
                panic!("Opening tag {} does not match closing tag {}", opening_tag.content, closing_tag.content);
            }
        }
        create_element(opening_tag.content.to_owned(), children)
    }

    fn parse_element_content(&mut self) -> Vec<XmlNode> {
        let mut content = vec![];
        while self.peek().token_type != ClosingTag {
            content.push(match self.peek() {
                XmlToken { token_type: XmlTokenType::Text, content:_ } => create_text(self.next().content),
                XmlToken { token_type: XmlTokenType::OpeningTag, content } |
                XmlToken { token_type: XmlTokenType::EmptyElementTag, content } => self.parse_element(),
                _ => panic!("Unknown token type")
            });
        }
        content
    }
}