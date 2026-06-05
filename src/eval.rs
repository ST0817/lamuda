use std::fmt::{self, Display, Formatter};

use crate::parse::Syntax;

pub enum Term {
    Unit,
    Int { value: i32 },
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "unit"),
            Self::Int { value } => write!(f, "{value}"),
        }
    }
}

pub fn eval(term: &Syntax) -> Term {
    match term {
        Syntax::Unit => Term::Unit,
        Syntax::Int { value } => Term::Int { value: *value },
    }
}
