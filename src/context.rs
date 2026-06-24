use std::{collections::HashMap, rc::Rc};

use crate::term::Term;

#[derive(Debug)]
pub struct RecRule {
    pub ctors: Vec<String>,
}

#[derive(Debug)]
pub enum ConstValue {
    Def { value: Rc<Term> },
    Ind,
    Ctor,
    Rec { rule: RecRule },
}

#[derive(Debug)]
pub struct Const {
    pub typ: Rc<Term>,
    pub value: ConstValue,
}

#[derive(Debug)]
pub struct GlobalContext {
    consts: HashMap<String, Const>,
}

impl GlobalContext {
    pub fn new() -> Self {
        Self {
            consts: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, cnst: Const) {
        self.consts.insert(name, cnst);
    }

    pub fn get(&self, name: &str) -> Option<&Const> {
        self.consts.get(name)
    }
}

#[derive(Clone, Debug)]
pub struct Var {
    pub name: String,
    pub typ: Rc<Term>,
}

#[derive(Clone, Debug)]
pub struct LocalContext {
    vars: Vec<Var>,
}

impl LocalContext {
    pub fn new() -> Self {
        Self { vars: Vec::new() }
    }

    pub fn get(&self, name: &str) -> Option<(usize, &Var)> {
        self.vars
            .iter()
            .rev()
            .enumerate()
            .find(|(_, entry)| entry.name == name)
    }

    pub fn push(&mut self, entry: Var) {
        self.vars.push(entry);
    }

    pub fn extend(&self, entry: Var) -> Self {
        let mut new_context = self.clone();
        new_context.push(entry);
        new_context
    }
}
