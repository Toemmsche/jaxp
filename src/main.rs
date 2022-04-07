use std::fs;
use std::str::FromStr;
use std::time::Instant;

use xmlparser::Token;

use crate::charstream::CharStream;
use crate::dfa::DFA;
use crate::parse::XmlParser;

mod dfa;
mod charstream;
mod xmlchar;
mod parse;


fn main() {
    let xml = fs::read_to_string("large.xml").unwrap();

    // Bench against xmlparser
    let mut now = Instant::now();
    let tree = roxmltree::Document::parse(&xml).unwrap();
    let elapsed = now.elapsed();
    println!("{:?}", tree.root());
    println!("Elapsed for xmlparser: {:.2?}", elapsed);


    let mut now = Instant::now();
    let mut parser = XmlParser {};
    let parsed = parser.parse(&xml);
    let elapsed = now.elapsed();
    println!("Overall parsing took: {:.2?}", elapsed);
    println!("{:?}", parsed.node_type);


    println!("Hello, world!");
}
