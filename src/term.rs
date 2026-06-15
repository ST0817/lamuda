use std::{
    fmt::{self, Display, Formatter},
    rc::Rc,
};

use ignorable::PartialEq;

use crate::env::Env;

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
        param_name: Rc<String>,
        param_type: Rc<Self>,
        body: Rc<Self>,
    },
    Prod {
        #[ignored(PartialEq)]
        param_name: Rc<String>,
        param_type: Rc<Self>,
        body_type: Rc<Self>,
    },
    Var {
        index: usize,
        #[ignored(PartialEq)]
        name: Rc<String>,
    },
    App {
        callee: Rc<Self>,
        arg: Rc<Self>,
    },
    Let {
        #[ignored(PartialEq)]
        name: Rc<String>,
        value: Rc<Self>,
        body: Rc<Self>,
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
            } => write!(f, "fun {param_name} : {param_type} => {body}"),
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
                    _ => write!(f, "{callee} ")?,
                }
                match arg.as_ref() {
                    Self::App { .. } => write!(f, "({arg})"),
                    _ => write!(f, "{arg}"),
                }
            }
            Self::Let { name, value, body } => write!(f, "let {name} := {value}; {body}"),
        }
    }
}

pub fn shift(term: &Rc<Term>, value: isize, cutoff: usize) -> Rc<Term> {
    match term.as_ref() {
        Term::Var { index, name } if *index >= cutoff => Rc::new(Term::Var {
            index: (*index as isize + value) as usize,
            name: name.clone(),
        }),
        Term::Fun {
            param_name,
            param_type,
            body,
        } => Rc::new(Term::Fun {
            param_name: param_name.clone(),
            param_type: shift(param_type, value, cutoff),
            body: shift(body, value, cutoff + 1),
        }),
        Term::Prod {
            param_name,
            param_type,
            body_type,
        } => Rc::new(Term::Prod {
            param_name: param_name.clone(),
            param_type: shift(param_type, value, cutoff),
            body_type: shift(body_type, value, cutoff + 1),
        }),
        Term::App { callee, arg } => Rc::new(Term::App {
            callee: shift(callee, value, cutoff),
            arg: shift(arg, value, cutoff),
        }),
        _ => term.clone(),
    }
}

pub fn normalize(term: &Rc<Term>, env: &Env) -> Rc<Term> {
    match term.as_ref() {
        Term::Var { index, .. } if let Some(value) = &env[*index] => {
            shift(value, *index as isize + 1, 0)
        }
        Term::Fun {
            param_name,
            param_type,
            body,
        } => {
            let norm_param_type = normalize(param_type, env);
            let new_env = env.extend(None);
            Rc::new(Term::Fun {
                param_name: param_name.clone(),
                param_type: norm_param_type,
                body: normalize(body, &new_env),
            })
        }
        Term::Prod {
            param_name,
            param_type,
            body_type,
        } => {
            let norm_param_type = normalize(param_type, env);
            let new_env = env.extend(None);
            Rc::new(Term::Prod {
                param_name: param_name.clone(),
                param_type: norm_param_type,
                body_type: normalize(body_type, &new_env),
            })
        }
        Term::App { callee, arg } => {
            let norm_callee = normalize(callee, env);
            let norm_arg = normalize(arg, env);
            match norm_callee.as_ref() {
                Term::Fun { body, .. } => {
                    let new_env = env.extend(Some(norm_arg));
                    shift(&normalize(body, &new_env), -1, 0)
                }
                _ => Rc::new(Term::App {
                    callee: norm_callee.clone(),
                    arg: norm_arg,
                }),
            }
        }
        Term::Let { value, body, .. } => {
            let norm_value = normalize(value, env);
            let new_env = env.extend(Some(norm_value));
            normalize(body, &new_env)
        }
        _ => term.clone(),
    }
}
