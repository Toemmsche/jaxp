use crate::token::default_tokenizer;
use std::fs;
use std::time::Instant;
use crate::parse::default_parser;

mod token;
mod parse;

fn main() {
    let mut now = Instant::now();

    let mut parser = default_parser();
    let xml = fs::read_to_string("large.xml").unwrap();
    let elem = parser.parse(xml.to_owned());
    //println!("{:?}", elem);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    // Bench against xmlparser
    now = Instant::now();
    let doc = roxmltree::Document::parse(&xml).unwrap();

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);




    println!("Hello, world!");
}
