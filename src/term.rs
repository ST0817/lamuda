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

pub fn subst_at(term1: &Rc<Term>, term2: &Rc<Term>, depth: usize) -> Rc<Term> {
    match term1.as_ref() {
        Term::Var { index, .. } if *index == depth => shift(term2, depth as isize, 0),
        Term::Var { index, name } if *index > depth => Rc::new(Term::Var {
            index: index - 1,
            name: name.clone(),
        }),
        Term::Fun {
            param_name,
            param_type,
            body,
        } => Rc::new(Term::Fun {
            param_name: param_name.clone(),
            param_type: subst_at(param_type, term2, depth),
            body: subst_at(body, term2, depth + 1),
        }),
        Term::Prod {
            param_name,
            param_type,
            body_type,
        } => Rc::new(Term::Prod {
            param_name: param_name.clone(),
            param_type: subst_at(param_type, term2, depth),
            body_type: subst_at(body_type, term2, depth + 1),
        }),
        Term::App { callee, arg } => Rc::new(Term::App {
            callee: subst_at(callee, term2, depth),
            arg: subst_at(arg, term2, depth),
        }),
        _ => term1.clone(),
    }
}

pub fn subst(term1: &Rc<Term>, term2: &Rc<Term>) -> Rc<Term> {
    subst_at(term1, term2, 0)
}

pub fn whnf(term: &Rc<Term>) -> Rc<Term> {
    match term.as_ref() {
        Term::App { callee, arg } => {
            let whnf_callee = whnf(callee);
            match whnf_callee.as_ref() {
                Term::Fun { body, .. } => whnf(&subst(body, arg)),
                _ => Rc::new(Term::App {
                    callee: whnf_callee,
                    arg: arg.clone(),
                }),
            }
        }
        _ => term.clone(),
    }
}

pub fn normalize(term: &Rc<Term>) -> Rc<Term> {
    match term.as_ref() {
        Term::Fun {
            param_name,
            param_type,
            body,
        } => Rc::new(Term::Fun {
            param_name: param_name.clone(),
            param_type: normalize(param_type),
            body: normalize(body),
        }),
        Term::Prod {
            param_name,
            param_type,
            body_type,
        } => Rc::new(Term::Prod {
            param_name: param_name.clone(),
            param_type: normalize(param_type),
            body_type: normalize(body_type),
        }),
        Term::App { callee, arg } => {
            let evaluated_callee = normalize(callee);
            match evaluated_callee.as_ref() {
                Term::Fun { body, .. } => normalize(&subst(body, &normalize(arg))),
                _ => Rc::new(Term::App {
                    callee: evaluated_callee.clone(),
                    arg: normalize(arg),
                }),
            }
        }
        _ => term.clone(),
    }
}
