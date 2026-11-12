use std::fmt::{self, Display, Formatter};

pub enum Expr {
    Int(i32),
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Expr::*;
        match self {
            Int(value) => value.fmt(f),
        }
    }
}
