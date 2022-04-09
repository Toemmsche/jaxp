#![feature(test)]
extern crate test;

use std::fs;
use std::str::FromStr;
use std::time::Instant;
use test::Bencher;
use roxmltree::{Descendants, Node};

use xmlparser::Token;

use crate::charstream::CharStream;
use crate::dfa::DFA;
use crate::parse::XmlParser;
use crate::tokenstream::TokenStream;

mod dfa;
mod charstream;
mod xmlchar;
mod parse;
mod tokenstream;


fn bench_my_tokenizer_slice_conv(xml: &str) {
    let now = Instant::now();
    for i in 0..100 {
        test::black_box(DFA {
            cs: CharStream { text: &xml, pos: 0 }
        }.tokenize().iter().map(|tok| &xml[tok.content.0..tok.content.1]));
    }
    println!("Elapsed for my_tokenize_convr: {:.2?}", now.elapsed());
}

fn bench_my_tokenizer(xml: &str) {
    let now = Instant::now();
    for i in 0..100 {
        test::black_box(DFA {
            cs: CharStream { text: &xml, pos: 0 }
        }.tokenize());
    }
    println!("Elapsed for my_tokenizer: {:.2?}", now.elapsed());
}


fn bench_my_parser(xml: &str) {
    let now = Instant::now();
    for i in 0..100 {
        let mut parser = XmlParser {ts: TokenStream{pos: 0, tokens: vec![]}};
        test::black_box(parser.parse(&xml));
    }
    println!("Elapsed for my_parser: {:.2?}", now.elapsed());
}


fn bench_roxmltree(xml: &str) {
    let now = Instant::now();
    for i in 0..100 {
        let tree = test::black_box(roxmltree::Document::parse(&xml).unwrap());
        let desc = test::black_box(tree.descendants().collect::<Vec<Node>>());
        println!("{:?}", desc);
    }
    println!("Elapsed for rxmltree: {:.2?}", now.elapsed());
}


fn bench_xmlparser(xml: &str) {
    let now = Instant::now();
    for i in 0..100 {
        let tokens = test::black_box(xmlparser::Tokenizer::from(&xml[..]).collect::<Vec<Result<Token, _>>>());
        println!("{:?}", tokens);
    }
    println!("Elapsed for xmlparser: {:.2?}", now.elapsed());
}

fn main() {
    let xml = &fs::read_to_string("test.xml").unwrap();
    bench_xmlparser(xml);
    bench_roxmltree(xml);
    println!("Hello, world!");
    bench_my_tokenizer_slice_conv(xml);
    bench_my_tokenizer(xml);
    bench_my_parser(xml);
}
