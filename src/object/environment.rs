use std::collections::HashMap;
use std::rc::Rc;
use crate::object::object::Object;

#[derive(PartialEq, Debug, Clone)]
pub struct Environment {
    // store: HashMap<String, Object>,
    store: HashMap<Rc<str>, Object>,
    outer: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            store: Default::default(),
            outer: None,
        }
    }

    // pub fn get(&self, name: &String) -> Option<Object> {
    pub fn get(&self, name: &Rc<str>) -> Option<Object> {
        // let store = self.store.borrow();

        match (self.store.get(name), &self.outer) {
            (result @ Some(_), _) => result.map(|object| object.clone()),
            (None, Some(outer)) => outer.get(name),
            (None, None) => None,
        }
    }

    pub fn set(&mut self, name: Rc<str>, object: Object) {
        // self.store.borrow_mut().insert(name, object);
        self.store.insert(name, object);
    }
}

impl From<Rc<Environment>> for Environment {

    fn from(value: Rc<Environment>) -> Self {
        Environment {
            store: Default::default(),
            outer: Some(value),
        }
    }
}