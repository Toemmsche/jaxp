use std::detect::__is_feature_detected::adx;
use std::str::{from_utf8, FromStr};

pub struct CharStream<'a> {
    pub(crate) pos: usize,
    pub(crate) slice: &'a str,
}

pub enum XmlChar {
    Whitespace()
}

impl<'a> CharStream<'a> {
    pub fn advance(& mut self, i: usize) -> & str {
        self.pos += i;
        self.slice_from(self.pos - i)
    }

    pub fn advance_until_byte(& mut self, byte: u8) -> & str {
        let from_pos = self.pos;
        while self.peek_byte() != byte {
            self.advance(1);
        }
        &self.slice[from_pos..self.pos]
    }

    pub fn slice_from(&self, from: usize) -> & str {
        &self.slice[from..self.pos]
    }

    pub fn peek_byte(&self) -> u8 {
        self.slice.as_bytes()[self.pos]
    }

    pub fn next_byte(&mut self) -> u8 {
        self.advance(1).as_bytes()[0]
    }

    pub fn has_next_byte(&self) -> bool {
        self.pos < self.slice.len()
    }

    pub fn expect(&mut self, expected: &str) {
        let actual = self.advance(expected.len());
        if expected != actual {
            panic!();
        }
    }

    pub fn upcoming(&mut self, test: &str) -> bool {
        &self.slice[self.pos..self.pos + test.len()] == test
    }

    pub fn next_char(& mut self) -> & [u8] {
        let prev_boundary = self.pos;
        self.advance(1);
        while self.slice.is_char_boundary(self.pos) {
            self.advance(1);
        }
        &self.slice.as_bytes()[prev_boundary..self.pos]
    }

    pub fn skip_spaces(&mut self) {
        while match self.peek_byte() {
            b' ' | b'\n' | b'\t' | b'\t' => {
                self.advance(1);
                true
            }
            _ => false
        } {};
    }

    pub fn consume_name(& mut self) -> & str {
        let from_pos = self.pos;
        while match self.peek_byte() {
            b if b >= b'A' && b <= b'z' => {
                self.advance(1);
                true
            }
            _ => false
        } {};
        // Consume at least one name token
        if from_pos == self.pos {
            panic!("Not consumed");
        }
        &self.slice[from_pos..self.pos].to_owned()
    }
}
