use std::rc::Rc;
use crate::ast::ast::{AST, Node};

#[derive(Debug)]
pub enum Expression {
    // Let(Box<str>, Box<Node>),
    Let(Box<Node>, Box<Node>),
    Identifier(Rc<str>),
    Integer(i64),
    Boolean(bool),
    String(Box<str>),
}

impl Expression {

    pub fn string(&self, literal: &str) -> Box<str> {
        match self {
            Expression::Let(name, value) => format!("{literal} {} = {}", name.string(), value.string()),
            _ => String::new(), //FIXME Remove
        }.into_boxed_str()
    }
}

struct Identifier(str);