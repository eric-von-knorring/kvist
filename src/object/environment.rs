use std::collections::HashMap;
use std::rc::Rc;
use crate::object::object::Object;

#[derive(PartialEq, Debug, Clone)]
pub struct Environment {
    store: HashMap<String, Object>,
    outer: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            store: Default::default(),
            outer: None,
        }
    }
}
