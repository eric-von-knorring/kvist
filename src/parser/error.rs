use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ParseError {
    pub col: u32,
    pub row: u32,
    pub message: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Row {}, Col: {}: {}", self.row, self.col, self.message)
    }
}