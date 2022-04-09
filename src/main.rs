#![feature(test)]
extern crate test;

use std::fs;
use std::str::FromStr;
use std::time::Instant;
use test::Bencher;

use xmlparser::Token;

use crate::charstream::CharStream;
use crate::dfa::DFA;
use crate::parse::XmlParser;

mod dfa;
mod charstream;
mod xmlchar;
mod parse;
mod tokenstream;


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
        let parser = XmlParser {};
        test::black_box(parser.parse(&xml));
    }
    println!("Elapsed for my_parser: {:.2?}", now.elapsed());
}


fn bench_roxmltree(xml: &str) {
    let now = Instant::now();
    for i in 0..100 {
        let tree = test::black_box(roxmltree::Document::parse(&xml).unwrap());
        test::black_box(tree.descendants());
    }
    println!("Elapsed for rxmltree: {:.2?}", now.elapsed());
}


fn bench_xmlparser(xml: &str) {
    let now = Instant::now();
    for i in 0..100 {
        test::black_box(xmlparser::Tokenizer::from(&xml[..]).collect::<Vec<Result<Token, _>>>());
    }
    println!("Elapsed for xmlparser: {:.2?}", now.elapsed());
}

fn main() {
    let xml = &fs::read_to_string("large.xml").unwrap();
    println!("Hello, world!");
    bench_my_tokenizer(xml);
    bench_my_parser(xml);
    bench_xmlparser(xml);
    bench_roxmltree(xml);
}
