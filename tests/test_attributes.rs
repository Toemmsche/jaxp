use jaxp_rust::error::XmlError::*;
use jaxp_rust::node::XmlNode;
use jaxp_rust::parse::XmlParser;

#[test]
pub fn test_single() {
    let xml = "<root attr=\"value\"></root>";
    let root_elem = XmlNode::ElementNode {
        name: "root",
        children: vec![
            XmlNode::AttributeNode {
                name: "attr",
                value: "value",
            },
        ],
    };
    assert_eq!(root_elem, XmlParser::default().parse(xml).unwrap());
}

#[test]
pub fn test_multiple() {
    let xml = "<root attr1=\"value1\" attr2=\"value2\" attr3=\"value3\"></root>";
    let root_elem = XmlNode::ElementNode {
        name: "root",
        children: vec![
            XmlNode::AttributeNode {
                name: "attr1",
                value: "value1",
            },
            XmlNode::AttributeNode {
                name: "attr2",
                value: "value2",
            },
            XmlNode::AttributeNode {
                name: "attr3",
                value: "value3",
            },
        ],
    };
    assert_eq!(root_elem, XmlParser::default().parse(xml).unwrap());
}

#[test]
pub fn test_random_spaces() {
    let xml = "<root  \t\r\t \n  attr1=\"value1\"   \t\t \n attr2=\"value2\"  \n\r \n \n \n \n    ></root    >";
    let root_elem = XmlNode::ElementNode {
        name: "root",
        children: vec![
            XmlNode::AttributeNode {
                name: "attr1",
                value: "value1",
            },
            XmlNode::AttributeNode {
                name: "attr2",
                value: "value2",
            },
        ],
    };
    assert_eq!(root_elem, XmlParser::default().parse(xml).unwrap());
}

#[test]
pub fn test_missing_spaces() {
    let xml = "<root  \t\r\t \n  attr1=\"value1\"attr2=\"value2\"  \n\r \n \n \n \n    ></root    >";
    let expected_err_target = "a".to_string();
    let actual_err = XmlParser::default().parse(&xml).unwrap_err();
    assert!(matches!(actual_err, IllegalToken{..})); // assert error type
    assert_eq!(expected_err_target, actual_err.get_target()); // assert target
}

#[test]
pub fn test_illegal_spaces() {
    let xml = "<root attr1=\n\"value1\"></root>";
    let expected_err_target = "\n".to_string();
    let actual_err = XmlParser::default().parse(&xml).unwrap_err();
    assert!(matches!(actual_err, IllegalToken{..})); // assert error type
    assert_eq!(expected_err_target, actual_err.get_target()); // assert target
}

#[test]
pub fn test_no_equality_sign() {
    let xml = "<root attr\"value\"></root>";
    let expected_err_target = "\"".to_string();
    let actual_err = XmlParser::default().parse(&xml).unwrap_err();
    assert!(matches!(actual_err, IllegalToken{..})); // assert error type
    assert_eq!(expected_err_target, actual_err.get_target()); // assert target
}

/// Valid names as defined in the standard. For more information, see
/// [here](https://www.w3.org/TR/xml/#sec-common-syn).
#[test]
pub fn test_valid_unicode_names() {
    // Start chars
    let mut start_chars_to_test = vec![":", "_", "Q", "z", "\u{FDF0}", "\u{EFFFF}", "\u{39F}", "\u{208F}", "😀"];
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
        let xml = format!("<root {}abc=\"value\"></root>", name);
        let expected_err_target = start_char;
        let actual_err = XmlParser::default().parse(&xml).unwrap_err();
        assert!(matches!(actual_err, IllegalToken{..})); // assert error type
        assert_eq!(expected_err_target, actual_err.get_target()); // assert target
    }

    for name_char in name_chars_to_test {
        let name = format!("ab{}c", name_char);
        let xml = format!("<root {}abc=\"value\"></root>", name);
        let expected_err_target = name_char;
        let actual_err = XmlParser::default().parse(&xml).unwrap_err();
        assert!(matches!(actual_err, IllegalToken{..})); // assert error type
        assert_eq!(expected_err_target, actual_err.get_target()); // assert target
    }
}


#[test]
pub fn test_invalid_attribute_value() {
    let chars_to_test = vec!["&", "<"];

    for char in chars_to_test {
        let name = format!("abc{}abc", char);
        let xml = format!("<root attr=\"{}\"></root>", name);
        // Finding any error is enough
        XmlParser::default().parse(&xml).unwrap_err();
    }
}

#[test]
pub fn test_single_quotes() {
    let single_qoutes = "<root attr='\"value\"'></root>";
    let root_elem = XmlNode::ElementNode {
        name: "root",
        children: vec![
            XmlNode::AttributeNode {
                name: "attr",
                value: "\"value\"",
            },
        ],
    };
    assert_eq!(root_elem, XmlParser::default().parse(&single_qoutes).unwrap());

    let invalid_quotes = "<root attr=`value`></root>";
    let expected_err_target = "`".to_string();
    let actual_err = XmlParser::default().parse(&invalid_quotes).unwrap_err();
    assert!(matches!(actual_err, IllegalToken{..})); // assert error type
    assert_eq!(expected_err_target, actual_err.get_target()); // assert target
}