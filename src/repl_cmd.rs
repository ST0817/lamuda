use crate::syntax::Syntax;

pub enum ReplCmd<'src> {
    Def {
        name: &'src str,
        value: Syntax<'src>,
    },
    Syntax {
        syntax: Syntax<'src>,
    },
}
