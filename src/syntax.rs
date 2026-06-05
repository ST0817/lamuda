use chumsky::span::Spanned;

use crate::typ::Type;

pub enum Syntax<'src> {
    Unit,
    Int {
        value: i32,
    },
    Fun {
        param_name: &'src str,
        param_type: Type,
        body: Box<Self>,
    },
    Var {
        name: Spanned<&'src str>,
    },
    App {
        callee: Spanned<Box<Self>>,
        arg: Spanned<Box<Self>>,
    },
}
