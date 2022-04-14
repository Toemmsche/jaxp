use crate::token::XmlRangeToken;

pub struct TokenStream {
    pub(crate) pos: usize,
    pub(crate) tokens: Vec<XmlRangeToken>,
}

impl Default for TokenStream {
    fn default() -> Self {
        TokenStream { pos: 0, tokens: vec![] }
    }
}

impl TokenStream {
    #[inline]
    pub fn next(&mut self) -> &XmlRangeToken {
        // cannot use peek() due to borrow checker
        let token = &self.tokens[self.pos];
        self.pos += 1;
        token
    }

    #[inline]
    pub fn peek(&self) -> &XmlRangeToken {
        &self.tokens[self.pos]
    }

    #[inline]
    pub fn peek_n(&self, n: usize) -> &XmlRangeToken {
        &self.tokens[self.pos + n]
    }
}