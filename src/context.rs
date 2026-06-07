use std::{ops::Index, rc::Rc};

use crate::term::Term;

#[derive(Clone, Debug)]
pub struct Context {
    map: Vec<(String, Rc<Term>)>,
}

impl Context {
    pub fn new() -> Self {
        Self { map: Vec::new() }
    }

    pub fn insert(&mut self, name: &str, term: Rc<Term>) {
        self.map.push((name.to_string(), term));
    }

    pub fn get(&self, name: &str) -> Option<(usize, &Rc<Term>)> {
        self.map
            .iter()
            .rev()
            .enumerate()
            .find(|(_, (var_name, _))| var_name == name)
            .map(|(index, (_, term))| (index, term))
    }
}

impl Context {
    pub fn extend(&self, name: &str, term: Rc<Term>) -> Self {
        let mut new_context = self.clone();
        new_context.insert(name, term);
        new_context
    }
}

impl Index<usize> for Context {
    type Output = Rc<Term>;

    fn index(&self, index: usize) -> &Self::Output {
        let (_, value) = &self.map[self.map.len() - 1 - index];
        value
    }
}
