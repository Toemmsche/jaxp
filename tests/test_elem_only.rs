extern crate core;

use jaxp_rust::error::XmlError::*;
use jaxp_rust::node::XmlNode;
use jaxp_rust::parse::XmlParser;

#[test]
pub fn test_root_only() {
    let xml = "<root></root>";
    let root_elem = XmlNode::ElementNode { name: "root", children: vec![] };
    assert_eq!(root_elem, XmlParser::default().parse(xml).unwrap());
}

#[test]
pub fn test_child_list() {
    let xml = "<root><a></a><b></b><c></c><d></d></root>";
    let root_elem = XmlNode::ElementNode {
        name: "root",
        children: vec![
            XmlNode::ElementNode { name: "a", children: vec![] },
            XmlNode::ElementNode { name: "b", children: vec![] },
            XmlNode::ElementNode { name: "c", children: vec![] },
            XmlNode::ElementNode { name: "d", children: vec![] },
        ],
    };
    assert_eq!(root_elem, XmlParser::default().parse(xml).unwrap());
}

#[test]
pub fn test_nested_structure() {
    let xml = "<root><a><b></b><c><d></d></c></a><e></e></root>";
    let root_elem = XmlNode::ElementNode {
        name: "root",
        children: vec![
            XmlNode::ElementNode {
                name: "a",
                children: vec![
                    XmlNode::ElementNode { name: "b", children: vec![] },
                    XmlNode::ElementNode {
                        name: "c",
                        children: vec![
                            XmlNode::ElementNode { name: "d", children: vec![] }
                        ],
                    },
                ],
            },
            XmlNode::ElementNode { name: "e", children: vec![] },
        ],
    };
    assert_eq!(root_elem, XmlParser::default().parse(xml).unwrap());
}

#[test]
pub fn test_empty_element_tag() {
    let xml = "<root><a/><b><c/></b></root>";
    let root_elem = XmlNode::ElementNode {
        name: "root",
        children: vec![
            XmlNode::ElementNode {
                name: "a",
                children: vec![],
            },
            XmlNode::ElementNode {
                name: "b",
                children: vec![
                    XmlNode::ElementNode {
                        name: "c",
                        children: vec![],
                    }
                ],
            },
        ],
    };
    assert_eq!(root_elem, XmlParser::default().parse(xml).unwrap());
}

#[test]
pub fn test_random_spaces() {
    let xml = "<root     \t\r\t \n   ><a    \t\r\t   /><b  \t  \n><c   \t\r\t /></b \n\n ></root  \n\n     \t\r\t  >";
    let root_elem = XmlNode::ElementNode {
        name: "root",
        children: vec![
            XmlNode::ElementNode {
                name: "a",
                children: vec![],
            },
            XmlNode::ElementNode {
                name: "b",
                children: vec![
                    XmlNode::ElementNode {
                        name: "c",
                        children: vec![],
                    }
                ],
            },
        ],
    };
    assert_eq!(root_elem, XmlParser::default().parse(xml).unwrap());
}

#[test]
pub fn test_illegal_spaces() {
    let xml = "<root><   /root>";
    let expected_err_target = " ".to_string();
    let actual_err = XmlParser::default().parse(&xml).unwrap_err();
    assert!(matches!(actual_err, IllegalToken{..})); // assert error type
    assert_eq!(expected_err_target, actual_err.get_target()); // assert target

    let xml = "<\nroot></root>";
    let expected_err_target = "\n".to_string();
    let actual_err = XmlParser::default().parse(&xml).unwrap_err();
    assert!(matches!(actual_err, IllegalToken{..})); // assert error type
    assert_eq!(expected_err_target, actual_err.get_target()); // assert target
}


/// Valid names as defined in the standard. For more information, see
/// [here](https://www.w3.org/TR/xml/#sec-common-syn).
#[test]
pub fn test_valid_unicode_names() {
    // Start chars
    let mut start_chars_to_test = vec![":", "_", "Q", "z", "\u{FDF0}", "\u{EFFFF}", "\u{39F}", "\u{208F}",  "😀"];
    // name chars
    // valid name chars include name start chars
    let mut name_chars_to_test = vec!["0", "3", "-", ".", "\u{B7}", "\u{0300}", "\u{203F}", "\u{1FFF}"];
    name_chars_to_test.append(&mut start_chars_to_test);

    for start_char in start_chars_to_test {
        let name = format!("{}abc", start_char);
        let xml = format!("<{}></{}>", name, name);
        let root_elem = XmlNode::ElementNode { name: &name, children: vec![] };
        assert_eq!(root_elem, XmlParser::default().parse(&xml).unwrap());
    }

    for name_char in name_chars_to_test {
        let name = format!("a{}{}", name_char, name_char);
        let xml = format!("<{}></{}>", name, name);
        let root_elem = XmlNode::ElementNode { name: &name, children: vec![] };
        assert_eq!(root_elem, XmlParser::default().parse(&xml).unwrap());
    }
}

#[test]
pub fn test_invalid_unicode_names() {
    let mut start_chars_to_test = vec!["-", ".", "$", "\u{200E}"];
    // name chars
    let mut name_chars_to_test = vec!["\u{B8}"];

    for start_char in start_chars_to_test {
        let name = format!("{}abc", start_char);
        let xml = format!("<{}></{}>", name, name);
        let expected_err_target = start_char;
        let actual_err = XmlParser::default().parse(&xml).unwrap_err();
        assert!(matches!(actual_err, IllegalToken{..})); // assert error type
        assert_eq!(expected_err_target, actual_err.get_target()); // assert target
    }

    for name_char in name_chars_to_test {
        let name = format!("ab{}c", name_char);
        let xml = format!("<{}></{}>", name, name);
        let expected_err_target = name_char;
        let actual_err = XmlParser::default().parse(&xml).unwrap_err();
        assert!(matches!(actual_err, IllegalToken{..})); // assert error type
        assert_eq!(expected_err_target, actual_err.get_target()); // assert target
    }
}

#[test]
pub fn test_non_matching_tags() {
    // Opening tag "a" does not match closing tag "aa"
    let xml = "<root><a></b></aa></root>";
    let expected_err_target = "aa".to_string();
    let actual_err = XmlParser::default().parse(xml).unwrap_err();
    assert!(matches!(actual_err, NonMatchingTags{ .. })); // assert error type
    assert_eq!(expected_err_target, actual_err.get_target()); // assert target
}
