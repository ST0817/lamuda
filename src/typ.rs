use std::fmt::Display;

#[derive(Clone, PartialEq)]
pub enum Type {
    Unit,
    Int,
    Fun {
        param_type: Box<Self>,
        body_type: Box<Self>,
    },
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit => write!(f, "Unit"),
            Self::Int => write!(f, "Int"),
            Self::Fun {
                param_type,
                body_type,
            } => match **param_type {
                Self::Fun { .. } => write!(f, "({param_type}) -> {body_type}"),
                _ => write!(f, "{param_type} -> {body_type}"),
            },
        }
    }
}
