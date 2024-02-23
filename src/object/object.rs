use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    String(Rc<str>),
    Array(Rc<[Object]>),
    // Null,
    Undefined,
}

pub(crate) trait Viewable {
    fn view(&self) -> String;
}

impl Viewable for Object {
    fn view(&self) -> String {
        match self {
            Object::Integer(value) => format!("{}", value),
            Object::Boolean(value) => format!("{}", value),
            Object::String(value) => format!("{}", value),
            Object::Array(values) => format!("[{}]", values.iter()
                .map(|object | object.view())
                .reduce(|acc, c| acc + ", " + &c)
                .unwrap_or("".to_string())
            ),
            // Object::Null => "null".to_string(),
            Object::Undefined => "undefined".to_string(),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Integer(_) => write!(f, "Integer"),
            Object::Boolean(_) => write!(f, "Boolean"),
            Object::String(_) => write!(f, "String"),
            Object::Array(_) => write!(f, "Array"),
            // Object::Null => write!(f, "Null"),
            Object::Undefined => write!(f, "Undefined"),
        }
    }
}
