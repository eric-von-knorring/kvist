use std::collections::HashMap;
use std::rc::Rc;
use crate::object::object::Object;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Environment {
    store: HashMap<String, Object>,
    outer: Option<Rc<Environment>>,
}
