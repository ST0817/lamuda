use std::{
    fmt::{self, Display, Formatter},
    rc::Rc,
};

use ignorable::PartialEq;

#[derive(Clone, Debug, PartialEq)]
pub enum Term {
    Sort {
        level: usize,
    },
    Unit,
    UnitType,
    Nat {
        value: usize,
    },
    NatType,
    Fun {
        #[ignored(PartialEq)]
        param_name: String,
        param_type: Rc<Self>,
        body: Rc<Self>,
    },
    Prod {
        #[ignored(PartialEq)]
        param_name: String,
        param_type: Rc<Self>,
        body_type: Rc<Self>,
    },
    Var {
        index: usize,
        #[ignored(PartialEq)]
        name: String,
    },
    App {
        callee: Rc<Self>,
        arg: Rc<Self>,
    },
    Let {
        #[ignored(PartialEq)]
        name: String,
        value: Rc<Self>,
        body: Rc<Self>,
    },
    Const {
        name: String,
    },
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Sort { level: 0 } => write!(f, "Prop"),
            Self::Sort { level: 1 } => write!(f, "Type"),
            Self::Sort { level } => write!(f, "Type {}", level - 1),
            Self::Unit => write!(f, "unit"),
            Self::UnitType => write!(f, "Unit"),
            Self::Nat { value } => write!(f, "{value}"),
            Self::NatType => write!(f, "Nat"),
            Self::Fun {
                param_name,
                param_type,
                body,
            } => write!(f, "fun ({param_name} : {param_type}) => {body}"),
            Self::Prod {
                param_name,
                param_type,
                body_type,
            } => {
                if param_name.is_empty() {
                    match **param_type {
                        Self::Prod { .. } => write!(f, "({param_type}) -> {body_type}"),
                        _ => write!(f, "{param_type} -> {body_type}"),
                    }
                } else {
                    write!(f, "({param_name} : {param_type}) -> {body_type}")
                }
            }
            Self::Var { index, name } => write!(f, "{index}#{name}"),
            Self::App { callee, arg } => {
                match callee.as_ref() {
                    Self::Fun { .. } => write!(f, "({callee}) ")?,
                    Self::App { callee, arg } if let Term::Fun { .. } = arg.as_ref() => {
                        write!(f, "{callee} ({arg}) ")?
                    }
                    _ => write!(f, "{callee} ")?,
                }
                match arg.as_ref() {
                    Self::App { .. } => write!(f, "({arg})"),
                    _ => write!(f, "{arg}"),
                }
            }
            Self::Let { name, value, body } => write!(f, "let {name} := {value}; {body}"),
            Self::Const { name } => write!(f, "{name}"),
        }
    }
}
