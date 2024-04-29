use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::ast::ast::Node;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Unit,
    // Integer(i64),
    Integer(i32),
    Float(f64),
    Boolean(bool),
    String(Rc<str>),
    Array(Rc<[Object]>),
    Function(Rc<[Node]>, Rc<Node>),
    // Null,
    Undefined,
}

pub(crate) trait Viewable {
    fn view(&self) -> String;
}

impl Viewable for Object {
    fn view(&self) -> String {
        match self {
            Object::Unit => "()".to_string(),
            Object::Integer(value) => format!("{}", value),
            Object::Float(value) => format!("{}", value),
            Object::Boolean(value) => format!("{}", value),
            Object::String(value) => format!("{}", value),
            Object::Array(values) => format!("[{}]", values.iter()
                .map(|object | object.view())
                // .reduce(|acc, c| acc + ", " + &c)
                .reduce(|acc, c| acc + " " + &c)
                .unwrap_or("".to_string())
            ),
            // Object::Null => "null".to_string(),
            Object::Undefined => "undefined".to_string(),
            // TODO proper formatted viewable
            Object::Function(_, _) => "(fn)".to_string()
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Unit => write!(f, "Unit"),
            Object::Integer(_) => write!(f, "Integer"),
            Object::Float(_) => write!(f, "Float"),
            Object::Boolean(_) => write!(f, "Boolean"),
            Object::String(_) => write!(f, "String"),
            Object::Array(_) => write!(f, "Array"),
            // Object::Null => write!(f, "Null"),
            Object::Undefined => write!(f, "Undefined"),
            Object::Function(_, _) => write!(f, "Function"),
        }
    }
}
