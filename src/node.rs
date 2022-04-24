#[derive(Debug, PartialEq)]
pub enum XmlNode<'a> {
    TextNode(&'a str),
    CommentNode(&'a str),
    ElementNode { name: &'a str, children: Vec<XmlNode<'a>> },
    AttributeNode { name: &'a str, value: &'a str },
    CdataSectionNode(&'a str),
    ProcessingInstructionNode(&'a str, Option<&'a str>),
}