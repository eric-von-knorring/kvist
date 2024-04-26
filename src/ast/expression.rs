use std::rc::Rc;

use crate::ast::ast::Node;

#[derive(Debug, PartialEq)]
pub enum Expression {
    SExpression(Box<[Node]>),
    // Set(Box<Node>, Box<Node>),
    Set(Rc<[(Node, Node)]>),
    Identifier(Rc<str>),
    Integer(i32),
    Float(f64),
    Boolean(bool),
    String(Rc<str>),
    Array(Box<[Node]>),
    Index(Box<Node>, Box<Node>),
    Prefix(Rc<str>, Box<[Node]>),
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    While(Box<Node>, Option<Box<Node>>),
}

impl Expression {

    pub fn string(&self, _literal: &str) -> Box<str> {
        match self {
            // Expression::Set(name, value) => format!("{literal} {} = {}", name.string(), value.string()),
            _ => String::new(), //FIXME Remove
        }.into_boxed_str()
    }
}

// struct Identifier(str);