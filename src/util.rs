use std::num::ParseIntError;

use crate::xmlchar::XmlChar;

pub fn decode_hex(reference: &str) -> Option<char> {
    let byte_vec: Vec<Result<u8, ParseIntError>> = (0..reference.len())
        .step_by(2)
        .map(|i| if i == reference.len() - 1 {
            u8::from_str_radix(&reference[i..], 16)
        } else {
            u8::from_str_radix(&reference[i..i + 2], 16)
        })
        .collect();
    // u32 can be constructed with 1-4 bytes
    return if byte_vec.len() > 4 || byte_vec.is_empty() {
        None
    } else {
        let mut res: u32 = 0;
        for i in 0..byte_vec.len() {
            res  = res << 8;
            match byte_vec[i] {
                Err(_) => return None,
                Ok(byte) => res += byte as u32
            };
        }
        let c = char::from_u32(res)?;
        if !c.is_xml_char() {
            return None;
        }
        Some(c)
    };
}