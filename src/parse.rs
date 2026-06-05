use chumsky::{
    Parser,
    extra::Err,
    prelude::choice,
    text::{self, ascii::keyword},
};

use crate::Error;

pub enum Syntax {
    Unit,
    Int { value: i32 },
}

fn int<'src>() -> impl Parser<'src, &'src str, i32, Err<Error<'src>>> {
    text::int(10).from_str().unwrapped()
}

fn unit_syntax<'src>() -> impl Parser<'src, &'src str, Syntax, Err<Error<'src>>> {
    keyword("unit").map(|_| Syntax::Unit)
}

fn int_syntax<'src>() -> impl Parser<'src, &'src str, Syntax, Err<Error<'src>>> {
    int().map(|value| Syntax::Int { value })
}

pub fn syntax<'src>() -> impl Parser<'src, &'src str, Syntax, Err<Error<'src>>> {
    choice((unit_syntax(), int_syntax()))
}
