
use std::fs;
use std::str::FromStr;
use std::time::Instant;
use crate::bytestream::CharStream;
use crate::dfa::DFA;

mod dfa;
mod bytestream;


fn main() {


    let xml = fs::read_to_string("large.xml").unwrap();

    let mut now = Instant::now();
    let mut tokenizer = DFA{
        cs: CharStream{pos: 0, slice: &xml}
    };
    tokenizer.tokenize(&xml);
    // Bench against xmlparser
    now = Instant::now();

    let mut t = None;
    for token in xmlparser::Tokenizer::from(&xml[..]) {
        //println!("{:?}", token);
        t = Some(token.unwrap());
    }
    println!("{:?}", t);
    let elapsed = now.elapsed();
    println!("Elapsed for xmlparser: {:.2?}", elapsed);




    println!("Hello, world!");
}
