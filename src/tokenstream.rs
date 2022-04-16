use crate::token::XmlToken;

pub struct TokenStream<'a> {
    pos: usize,
    tokens: Vec<XmlToken<'a>>,
}

impl Default for TokenStream<'_> {
    fn default() -> Self {
        TokenStream { pos: 0, tokens: vec![] }
    }
}

impl<'a> From<Vec<XmlToken<'a>>> for TokenStream<'a> {
    fn from(tokens: Vec<XmlToken<'a>>) -> Self {
        TokenStream { pos: 0, tokens }
    }
}

impl<'a> TokenStream<'a> {

    pub fn next(&mut self) -> &XmlToken<'a> {
        // cannot use peek() due to borrow checker
        let token = &self.tokens[self.pos];
        self.pos += 1;
        token
    }


    pub fn peek(&self) -> &XmlToken {
        &self.tokens[self.pos]
    }


    pub fn peek_n(&self, n: usize) -> &XmlToken<'a> {
        &self.tokens[self.pos + n]
    }


    pub fn has_next(&self) -> bool {
        self.pos < self.tokens.len()
    }
}