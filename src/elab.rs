use crate::{expr, syntax};

pub fn elab(stx: syntax::Expr) -> expr::Expr {
    use syntax::Expr::*;
    match stx {
        Int(value) => expr::Expr::Int(value),
    }
}
