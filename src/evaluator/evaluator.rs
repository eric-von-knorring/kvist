use std::rc::Rc;
use crate::ast::ast::{Node, Program};
use crate::ast::expression::Expression;
use crate::object::environment::Environment;
use crate::object::object::Object;

pub trait Eval {
    fn eval(&self, environment: &mut Environment) -> Result<Object, String>;

}

impl Eval for Program {
    fn eval(&self, environment: &mut Environment) -> Result<Object, String> {
        let mut result = Object::Integer(0);
        for node in self.nodes {
            result = node.eval()?;
        }
        result.into()
    }
}

impl Eval for Node {
    fn eval(&self, environment: &mut Environment) -> Result<Object, String> {
        match self.expression {
            Expression::Let(_, _) => eval_let("", environment),
            Expression::Identifier(value) => eval_identifier(&value, environment),
            Expression::Integer(value) => Object::Integer(value).into(),
            Expression::Boolean(value) => Object::Boolean(value).into(),
            Expression::String(value) => Object::String(value.clone()).into(),
            // Expression::Prefix(operator, operands) => eval_prefix_expression(&operator, prefix.right.eval(environment)?),
            Expression::Prefix(operator, operands) => eval_prefix_expression(&operator, &operands),
        }
    }
}



fn eval_let(identifier: &str, environment: &mut Environment) -> Result<Object, String> {
    todo!()
}

fn eval_identifier(identifier: &str, environment: &mut Environment) -> Result<Object, String> {
    // if let Some(value) = environment.get(&identifier.value) {
    //     return Ok(value);
    // }
    // if let Some(value) = builtins(identifier.value.as_str()) {
    //     return Ok(value);
    // };
    // return Err(format!("No binding for identifier '{}'", identifier.value));
    todo!()
}

fn eval_prefix_expression(operator: &Rc<str>, operand: &[Node]) -> Result<Object, String> {
    todo!()
}
