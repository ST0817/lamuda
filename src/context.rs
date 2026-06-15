use std::rc::Rc;

use crate::term::Term;

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: Rc<String>,
    pub typ: Rc<Term>,
}

#[derive(Clone, Debug)]
pub struct Context {
    map: Vec<Entry>,
}

impl Context {
    pub fn new() -> Self {
        Self { map: Vec::new() }
    }

    pub fn get(&self, name: &str) -> Option<(usize, &Entry)> {
        self.map
            .iter()
            .rev()
            .enumerate()
            .find(|(_, entry)| entry.name.as_ref() == name)
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
