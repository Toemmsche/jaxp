use crate::token::XmlToken;

pub struct TokenStream {
    pub(crate) pos: usize,
    pub(crate) tokens: Vec<XmlToken>,
}

impl TokenStream {
    #[inline]
    pub fn next(&mut self) -> &XmlToken {
        // cannot use peek() due to borrow checker
        let token = &self.tokens[self.pos];
        self.pos += 1;
        token
    }

    #[inline]
    pub fn peek(&self) -> &XmlToken {
        &self.tokens[self.pos]
    }

    #[inline]
    pub fn peek_n(&self, n: usize) -> &XmlToken {
        &self.tokens[self.pos + n]
    }
}