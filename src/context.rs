use std::{collections::HashMap, ops::Index};

pub struct Context<T> {
    map: HashMap<String, T>,
}

impl<T: Clone> Clone for Context<T> {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
        }
    }
}

impl<T> Context<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: &str, value: T) {
        self.map.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<&T> {
        self.map.get(name)
    }
}

impl<T: Clone> Context<T> {
    pub fn extend(&self, name: &str, value: T) -> Self {
        let mut new_context = self.clone();
        new_context.insert(name, value);
        new_context
    }
}

impl<T> Index<&str> for Context<T> {
    type Output = T;

    fn index(&self, index: &str) -> &Self::Output {
        &self.map[index]
    }
}
