#![feature(test)]
extern crate test;

use std::fs;
use std::time::Instant;

use xmlparser::Token;

use crate::parse::XmlParser;
use crate::tokenize::XmlTokenizer;

mod tokenize;
mod node;
mod chariter;
mod xmlchar;
mod parse;
mod tokenstream;
mod token;
mod error;
mod util;
mod textrange;


static LIMIT: usize = 1;

fn bench_my_tokenizer(xml: &str) {
    let now = Instant::now();
    for _ in 0..LIMIT {
        let tokens = test::black_box(XmlTokenizer::default()).tokenize(xml).unwrap();
        assert!(tokens.len() < 10000000);
        println!("{:?}", tokens);
    }
    println!("Elapsed for my_tokenizer: {:.2?}", now.elapsed());
}


fn bench_my_parser(xml: &str) {
    let now = Instant::now();
    for _ in 0..LIMIT {
        let mut parser = XmlParser::default();
        test::black_box(parser.parse(&xml).unwrap());
        //println!("{:?}", parsed);
    }
    println!("Elapsed for my_parser: {:.2?}", now.elapsed());
}


fn bench_roxmltree(xml: &str) {
    let now = Instant::now();
    for _ in 0..LIMIT {
        test::black_box(roxmltree::Document::parse(&xml).unwrap());
        //println!("{}", tree.descendants().collect::<Vec<Node>>().len());
    }
    println!("Elapsed for rxmltree: {:.2?}", now.elapsed());
}


fn bench_xmlparser(xml: &str) {
    let now = Instant::now();
    for _ in 0..LIMIT {
        let tokens = test::black_box(xmlparser::Tokenizer::from(&xml[..]).collect::<Vec<Result<Token, _>>>());
        assert!(tokens.len() < 10000000000);
        //print!("{:?}", tokens);
    }
    println!("Elapsed for xmlparser: {:.2?}", now.elapsed());
}

fn main() {
    let xml = &fs::read_to_string("test.xml").unwrap();

    bench_my_tokenizer(xml);

    bench_xmlparser(xml);
    bench_roxmltree(xml);


    bench_my_parser(xml);


    println!("Hello, world!");
}
