#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextRange<'a> {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) slice: &'a str
}

impl TextRange<'_> {
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}