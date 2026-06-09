use std::rc::Rc;

use crate::term::Term;

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub typ: Rc<Term>,
    pub value: Option<Rc<Term>>,
}

#[derive(Clone, Debug)]
pub struct Context {
    map: Vec<Entry>,
}

impl Context {
    pub fn new() -> Self {
        Self { map: Vec::new() }
    }

    fn get(&self, predicate: impl FnMut(&(usize, &Entry)) -> bool) -> Option<(usize, &Entry)> {
        self.map.iter().rev().enumerate().find(predicate)
    }

    pub fn get_name(&self, name: &str) -> Option<(usize, &Entry)> {
        self.get(|(_, entry)| entry.name == name)
    }

    pub fn get_index(&self, index: usize) -> Option<&Entry> {
        self.get(|(var_index, _)| *var_index == index)
            .map(|(_, entry)| entry)
    }

    pub fn push(&mut self, entry: Entry) {
        self.map.push(entry);
    }

    pub fn extend(&self, entry: Entry) -> Self {
        let mut new_context = self.clone();
        new_context.push(entry);
        new_context
    }
}
