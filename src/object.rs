use std::{
    fmt::{self, Display, Formatter},
    rc::Rc,
};

#[derive(Clone)]
pub enum Object {
    Unit,
    Int { value: i32 },
    Fun { fun: Rc<dyn Fn(Self) -> Self> },
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "unit"),
            Self::Int { value } => write!(f, "{value}"),
            Self::Fun { .. } => write!(f, "function"),
        }
    }
}
