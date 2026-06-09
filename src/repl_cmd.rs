use crate::syntax::Syntax;

pub enum ReplCmd<'src> {
    Def {
        name: &'src str,
        syntax: Syntax<'src>,
    },
    Syntax {
        syntax: Syntax<'src>,
    },
}
