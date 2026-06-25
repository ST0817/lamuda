use chumsky::span::Spanned;

#[derive(Clone, Debug)]
pub enum Syntax<'src> {
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
        param_name: &'src str,
        param_type: Spanned<Box<Self>>,
        body: Box<Self>,
    },
    Prod {
        param_name: &'src str,
        param_type: Spanned<Box<Self>>,
        body_type: Spanned<Box<Self>>,
    },
    Var {
        name: Spanned<&'src str>,
    },
    App {
        callee: Spanned<Box<Self>>,
        arg: Spanned<Box<Self>>,
    },
    Let {
        name: &'src str,
        value: Box<Self>,
        body: Box<Self>,
    },
}

#[derive(Debug)]
pub struct Ctor<'src> {
    pub name: Spanned<&'src str>,
    pub typ: Spanned<Syntax<'src>>,
}

#[derive(Debug)]
pub enum Command<'src> {
    Def {
        name: Spanned<&'src str>,
        value: Syntax<'src>,
    },
    Inductive {
        name: Spanned<&'src str>,
        typ: Spanned<Syntax<'src>>,
        ctors: Vec<Ctor<'src>>,
    },
}
