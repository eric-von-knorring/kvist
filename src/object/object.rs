use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::ast::ast::Node;
use crate::object::environment::Environment;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Unit,
    // Integer(i64),
    Integer(i32),
    Float(f64),
    Boolean(bool),
    String(Rc<str>),
    Array(Rc<[Object]>),
    Spread(Rc<[Object]>),
    Function(Rc<[Node]>, Rc<Option<Node>>, Rc<Node>, Rc<Environment>),
    Builtin(fn(Box<[Object]>) -> Result<Object, String>),
    // Null,
    Undefined,
}

impl Object {

    pub fn expand_spread(self, mut consumer: impl FnMut(Object) -> ()) {
        match self {
            Object::Spread(operand) => operand.iter()
                .for_each( |object| consumer(object.clone())),
            object @ _ => consumer(object),
        }
    }

    pub fn spread_to_single(self) -> Option<Object> {
        match self {
            Object::Spread(operand) => operand.last().map(|object| object.clone()),
            object @ _ => object.into(),
        }
    }
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
            Object::Function(_, _, _, _) => "(fn)".to_string(),
            Object::Builtin(_) => "(builtin)".to_string(),
            Object::Spread(values) => format!("..[{}]", values.iter()
                .map(|object | object.view())
                // .reduce(|acc, c| acc + ", " + &c)
                .reduce(|acc, c| acc + " " + &c)
                .unwrap_or("".to_string())
            ),
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
            Object::Function(_, _, _, _) => write!(f, "Function"),
            Object::Builtin(_) => write!(f, "Builtin"),
            Object::Spread(_) => write!(f, "Spread"),
        }
    }
}
