#![feature(test)]
extern crate test;

use std::convert::TryInto;
use std::fs;
use std::str::FromStr;
use std::time::Instant;
use test::Bencher;

use roxmltree::{Descendants, Node};
use xmlparser::Token;

use crate::charstream::CharIter;
use crate::node::XmlNode::ElementNode;
use crate::parse::XmlParser;
use crate::tokenize::XmlTokenizer;
use crate::tokenstream::TokenStream;

mod tokenize;
mod node;
mod charstream;
mod xmlchar;
mod parse;
mod tokenstream;
mod token;
mod xmlerror;
mod util;


static limit: usize = 100;

fn bench_my_tokenizer(xml: &str) {
    let now = Instant::now();
    for i in 0..limit {
        let tokens = test::black_box(XmlTokenizer::default()).tokenize(xml).unwrap();
        assert!(tokens.len() < 10000000);
        //println!("{:?}", tokens);
    }
    println!("Elapsed for my_tokenizer: {:.2?}", now.elapsed());
}


fn bench_my_parser(xml: &str) {
    let now = Instant::now();
    for i in 0..limit {
        let mut parser = XmlParser::default();
        let parsed = test::black_box(parser.parse(&xml).unwrap());
        //println!("{:?}", parsed);
        if let ElementNode { name, children } = parsed {
            assert_ne!(name, "asdfla");
        }
    }
    println!("Elapsed for my_parser: {:.2?}", now.elapsed());
}


fn bench_roxmltree(xml: &str) {
    let now = Instant::now();
    for i in 0..limit {
        let tree = test::black_box(roxmltree::Document::parse(&xml).unwrap());
        assert_ne!(tree.root().text(), Some("asdfla"));
    }
    println!("Elapsed for rxmltree: {:.2?}", now.elapsed());
}


fn bench_xmlparser(xml: &str) {
    let now = Instant::now();
    for i in 0..limit {
        let tokens = test::black_box(xmlparser::Tokenizer::from(&xml[..]).collect::<Vec<Result<Token, _>>>());
        assert!(tokens.len() < 10000000000);
        //print!("{:?}", tokens);
    }
    println!("Elapsed for xmlparser: {:.2?}", now.elapsed());
}

fn main() {
    let xml = &fs::read_to_string("large_pi.xml").unwrap();

    bench_roxmltree(xml);
    bench_xmlparser(xml);
    bench_my_parser(xml);
    bench_my_tokenizer(xml);



    println!("Hello, world!");
}
