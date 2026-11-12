use std::fmt::{self, Display, Formatter};

#[derive(Clone, PartialEq)]
pub enum Token {
    Int(i32),
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Token::*;
        match self {
            Int(value) => value.fmt(f),
        }
    }
}
