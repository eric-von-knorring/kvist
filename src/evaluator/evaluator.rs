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
        for node in self.nodes.iter() {
            // println!("{:?}", node);
            result = node.eval(environment)?;
        }
        result.into()
        // Ok(result)
    }
}

impl Eval for Node {
    fn eval(&self, environment: &mut Environment) -> Result<Object, String> {
        match &self.expression {
            Expression::SExpression(nodes) => eval_s_expression(nodes, environment),
            Expression::Let(_, _) => eval_let("", environment),
            Expression::Identifier(value) => eval_identifier(&value, environment),
            Expression::Integer(value) => Object::Integer(*value).into(),
            Expression::Float(value) => Object::Float(*value).into(),
            Expression::Boolean(value) => Object::Boolean(*value).into(),
            Expression::String(value) => Object::String(value.clone()).into(),
            // Expression::Prefix(operator, operands) => eval_prefix_expression(&operator, prefix.right.eval(environment)?),
            Expression::Prefix(operator, operands) => eval_prefix_expression(&operator, &operands, environment),
        }.map_err(|err| format!("row: {}, col: {}, {err}", self.token.row, self.token.col))
    }
}

fn eval_s_expression(nodes: &[Node], environment: &mut Environment) -> Result<Object, String> {
    let mut result = Object::Unit;
    for node in nodes {
        result = node.eval(environment)?;
    }
    return result.into();
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

fn eval_prefix_expression(operator: &Rc<str>, operands: &[Node], environment: &mut Environment) -> Result<Object, String> {
    match operator.as_ref() {
        // "+" => plus_operator(acc?, next?),
        "+" => plus_operator(operands, environment),
        "-" => minus_operator(operands, environment),
        "*" => multiply_operator(operands, environment),
        "/" => divide_operator(operands, environment),
        _ => Err(format!("unknown operator '{operator}'")),
    }
    // operands.iter()
    //     .map(|node| node.eval(environment))
    //     .reduce(|acc, next| {
    //     }).unwrap_or(Err("Could not eval operator".to_string()))
    // ;
    // todo!()
}


// fn plus_operator(left: Object, right: Object) -> Result<Object, String>{
fn plus_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, String>{
    if operands.len() == 1 {
        return operands[0].eval(environment);
    }
    // match (left, right) {
    //     (Object::Integer(left), Object::Integer(right)) => Object::Integer(left + right).into(),
    //     (_, _) => Err("Type mismatch".to_string()),
    // }
    operands.iter()
        .map(|node| node.eval(environment))
        .reduce(|left, right| {
            match (left?, right?) {
                (Object::Integer(left), Object::Integer(right)) => {Object::Integer(left + right).into()},

                (Object::Float(left), Object::Integer(right)) => Object::Float(left + f64::from(right)).into(),
                (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) + right).into(),
                (Object::Float(left), Object::Float(right)) => Object::Float(left + right).into(),

                (Object::String(left), Object::Integer(right)) => Object::String(format!("{left}{right}").into()).into(),
                (Object::Integer(left), Object::String(right)) => Object::String(format!("{left}{right}").into()).into(),
                (left @ _, right @ _) => Err(format!("Type mismatch (+ {left} {right})")),
            }
        }).unwrap_or(Err("Could not eval operator".to_string()))
}

fn minus_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, String>{
    if operands.len() == 1 {
        let object = operands[0].eval(environment)?;
        // return if let Object::Integer(value) = object {
        //     Object::Integer(0 - value).into()
        // } else { Err(format!("Type mismatch (- {object})")) };
        return match operands[0].eval(environment)? {
            Object::Integer(value) => Object::Integer(0 - value).into(),
            Object::Float(value) => Object::Float(0. - value).into(),
            _ => Err(format!("Type mismatch (- {object})")),
        }
    }
    operands.iter()
        .map(|node| node.eval(environment))
        .reduce(|left, right| {
            match (left?, right?) {
                (Object::Integer(left), Object::Integer(right)) => Object::Integer(left - right).into(),

                (Object::Float(left), Object::Integer(right)) => Object::Float(left - f64::from(right)).into(),
                (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) - right).into(),
                (Object::Float(left), Object::Float(right)) => Object::Float(left - right).into(),

                (left @ _, right @ _) => Err(format!("Type mismatch (- {left} {right})")),
            }
        }).unwrap_or(Err("Could not eval operator".to_string()))
}

fn multiply_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, String> {
    let mut product = Object::Integer(1);
    for operand in operands {
        product = match (product, operand.eval(environment)?) {
            (Object::Integer(left), Object::Integer(right)) => Object::Integer(left * right),
            (Object::Float(left), Object::Integer(right)) => Object::Float(left * f64::from(right)),
            (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) * right),
            (Object::Float(left), Object::Float(right)) => Object::Float(left * right),
            (left @ _, right @ _)  => return Err(format!("Type mismatch (* {left} {right})")),
        };
    }

    return product.into();
}

fn divide_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, String> {
    if operands.len() == 0 {
        return Object::Undefined.into();
    }
    if let Some(single) = operands.single() {
        return match single.eval(environment)? {
            Object::Integer(value) => Object::Float(1. / f64::from(value)).into(),
            Object::Float(value) => Object::Float(value).into(),
            object @ _ => Err(format!("Type mismatch (/ {object})"))
        }
    }

    let mut result = operands[0].eval(environment)?;
    for operand in operands[1..].iter() {
        result = match (result, operand.eval(environment)?) {
            // (Object::Integer(left), Object::Integer(right)) => Object::Integer(left / right),
            (Object::Integer(left), Object::Integer(right)) => {
                if left % right != 0 {
                    Object::Float(f64::from(left) / f64::from(right))
                } else {
                    Object::Integer(left / right)
                }
            },

            (Object::Float(left), Object::Integer(right)) => Object::Float(left / f64::from(right)),
            (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) / right),
            (Object::Float(left), Object::Float(right)) => Object::Float(left / right),
            (left @ _, right @ _)  => return Err(format!("Type mismatch (/ {left} {right})")),
        }
    }

    return result.into();
}

impl<T> From<Object> for Result<Object, T> {
    fn from(value: Object) -> Self {
        Ok(value)
    }
}

trait Single <T>{
    fn single(&self) -> Option<&T>;
}

// impl Single<Node> for [Node] {
impl<T> Single<T> for [T] {
    fn single(&self) -> Option<&T> {
        if self.len() == 1 {
            return self.get(0);
        }
        None
    }
}