extern crate core;

use jaxp_rust::error::XmlError::*;
use jaxp_rust::node::XmlNode;
use jaxp_rust::node::XmlNode::TextNode;
use jaxp_rust::parse::XmlParser;

#[test]
pub fn test_single() {
    let text = "this is some text";
    let xml = format!("<root>{}</root>", text);
    let root_elem = XmlNode::ElementNode { name: "root", children: vec![TextNode(text)] };
    assert_eq!(root_elem, XmlParser::default().parse(&xml).unwrap());
}


#[test]
pub fn test_nested() {
    let xml = "<root>root level<a>first level<b>second level</b>more first level</a>another root level</root>";
    let root_elem = XmlNode::ElementNode {
        name: "root",
        children: vec![
            XmlNode::TextNode("root level"),
            XmlNode::ElementNode {
                name: "a",
                children: vec![
                    XmlNode::TextNode("first level"),
                    XmlNode::ElementNode {
                        name: "b",
                        children: vec![
                            XmlNode::TextNode("second level")
                        ],
                    },
                    XmlNode::TextNode("more first level"),
                ],
            },
            XmlNode::TextNode("another root level"),
        ],
    };
    assert_eq!(root_elem, XmlParser::default().parse(xml).unwrap());
}

#[test]
pub fn test_spaces() {
    let xml = "<root>\r\n  <a>\n    indented text\n  </a></root>";
    let root_elem = XmlNode::ElementNode {
        name: "root",
        children: vec![
            XmlNode::TextNode("\r\n  "),
            XmlNode::ElementNode {
                name: "a",
                children: vec![
                    XmlNode::TextNode("\n    indented text\n  ")
                ],
            },
        ],
    };
    assert_eq!(root_elem, XmlParser::default().parse(xml).unwrap());
}

#[test]
pub fn test_valid_unicode() {
    let valid_text = "😀;->ä和製漢字";
    let xml = format!("<root>{}</root>", valid_text);
    let root_elem = XmlNode::ElementNode { name: "root", children: vec![TextNode(valid_text)] };
    assert_eq!(root_elem, XmlParser::default().parse(&xml).unwrap());
}

#[test]
pub fn test_invalid_unicode() {
    let illegal_texts = vec!["\u{FFFF}", "]]>", "&", "<"];
    for illegal_text in illegal_texts {
        let xml = format!("<root>{}</root>", illegal_text);
        // error type can vary
        let actual_err = XmlParser::default().parse(&xml).unwrap_err();
    }
}

//TODO test text before and after root element
