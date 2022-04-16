use crate::token::XmlRangeToken;

pub struct TokenStream {
    pos: usize,
    tokens: Vec<XmlRangeToken>,
}

impl Default for TokenStream {
    fn default() -> Self {
        TokenStream { pos: 0, tokens: vec![] }
    }
}

impl From<Vec<XmlRangeToken>> for TokenStream {
    fn from(tokens: Vec<XmlRangeToken>) -> Self {
        TokenStream { pos: 0, tokens }
    }
}

impl TokenStream {

    pub fn next(&mut self) -> &XmlRangeToken {
        // cannot use peek() due to borrow checker
        let token = &self.tokens[self.pos];
        self.pos += 1;
        token
    }


    pub fn peek(&self) -> &XmlRangeToken {
        &self.tokens[self.pos]
    }


    pub fn peek_n(&self, n: usize) -> &XmlRangeToken {
        &self.tokens[self.pos + n]
    }


    pub fn has_next(&self) -> bool {
        self.pos < self.tokens.len()
    }
}