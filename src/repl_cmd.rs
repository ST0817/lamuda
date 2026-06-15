use crate::syntax::{Command, Syntax};

pub enum ReplCmd<'src> {
    Command { command: Command<'src> },
    Syntax { syntax: Syntax<'src> },
}
