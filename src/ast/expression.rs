use std::rc::Rc;

use crate::ast::ast::{AST, Node};

#[derive(Debug, PartialEq)]
pub enum Expression {
    SExpression(Box<[Node]>),
    Let(Box<Node>, Box<Node>),
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

    pub fn string(&self, literal: &str) -> Box<str> {
        match self {
            Expression::Let(name, value) => format!("{literal} {} = {}", name.string(), value.string()),
            _ => String::new(), //FIXME Remove
        }.into_boxed_str()
    }
}

// struct Identifier(str);