use std::{ops::Index, rc::Rc};

use crate::term::Term;

#[derive(Clone, Debug)]
pub struct Env {
    terms: Vec<Option<Rc<Term>>>,
}

impl Env {
    pub fn new() -> Self {
        Self { terms: Vec::new() }
    }

    pub fn push(&mut self, term: Option<Rc<Term>>) {
        self.terms.push(term);
    }

    pub fn extend(&self, term: Option<Rc<Term>>) -> Self {
        let mut new_env = self.clone();
        new_env.push(term);
        new_env
    }
}

impl Index<usize> for Env {
    type Output = Option<Rc<Term>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.terms[self.terms.len() - 1 - index]
    }
}
