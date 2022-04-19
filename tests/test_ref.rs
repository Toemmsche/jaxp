extern crate core;

use jaxp_rust::node::XmlNode;
use jaxp_rust::parse::XmlParser;
use jaxp_rust::token::XmlToken;
use jaxp_rust::error::XmlError;

#[test]
pub fn test_valid_char_references() {
    let to_test = vec!["&amp;", "&lt;", "&gt;", "&quot;", "&apos;", "&#x9;","&#xA;", "&#xD;" , "&#x10FFFF;", "&#x9;", "&#10;", "&#13;", "&#32;"];
    for reference in to_test {
        let xml = format!("<root>{}</root>", reference);
        let root_elem = XmlNode::ElementNode { name: "root", children: vec![XmlNode::TextNode(reference)] };
        assert_eq!(root_elem, XmlParser::default().parse(&xml).unwrap());
    }
}


#[test]
pub fn test_invalid_char_references() {
    let to_test = vec![ "&unknown;" ,"&#xaaaaffffffff;", "&#x8;","&#a;", "&#10345672367;"];
    for reference in to_test {
        let expected_err_target = reference.to_string();
        let xml = format!("<root>&amp;{}</root>", expected_err_target);
        let actual_err_target = XmlParser::default().parse(&xml).unwrap_err().get_target();
        assert_eq!(expected_err_target, actual_err_target);
    }
}
