#![feature(test)]
extern crate test;

use std::convert::TryInto;
use std::fs;
use std::str::FromStr;
use std::time::Instant;
use test::Bencher;
use roxmltree::{Descendants, Node};

use xmlparser::Token;

use crate::charstream::CharStream;
use crate::dfa::DFA;
use crate::parse::XmlNode::ElementNode;
use crate::parse::XmlParser;
use crate::tokenstream::TokenStream;

mod dfa;
mod charstream;
mod xmlchar;
mod parse;
mod tokenstream;
mod token;
mod xmlerror;


static limit: usize = 100;

fn bench_my_tokenizer(xml: &str) {
    let now = Instant::now();
    for i in 0..limit {
        let tokens = test::black_box(DFA {
            cs: CharStream { text: &xml, pos: 0 }
        }.tokenize());
        assert!(tokens.len() < 10000000);
        //println!("{:?}", tokens);
    }
    println!("Elapsed for my_tokenizer: {:.2?}", now.elapsed());
}


fn bench_my_parser(xml: &str) {
    let now = Instant::now();
    for i in 0..limit {
        let mut parser = XmlParser {ts: TokenStream{pos: 0, tokens: vec![]}};
        let parsed = test::black_box(parser.parse(&xml));
        if let ElementNode {name, children} = parsed {
            assert_ne!(name, "asdfla");
        }
        //println!("{:?}", parsed);
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
    }
    println!("Elapsed for xmlparser: {:.2?}", now.elapsed());
}

fn main() {
    let xml = &fs::read_to_string("large.xml").unwrap();

    bench_my_parser(xml);
    bench_my_tokenizer(xml);
    bench_roxmltree(xml);
    bench_xmlparser(xml);

    println!("Hello, world!");

}
