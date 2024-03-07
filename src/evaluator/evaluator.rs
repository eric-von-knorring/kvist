use std::rc::Rc;

use crate::ast::ast::{Node, Program};
use crate::ast::expression::Expression;
use crate::object::environment::Environment;
use crate::object::object::{Object, Viewable};

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
            Expression::Let(identifier, value) => eval_let(identifier, value, environment),
            Expression::Identifier(value) => eval_identifier(value, environment),
            Expression::Integer(value) => Object::Integer(*value).into(),
            Expression::Float(value) => Object::Float(*value).into(),
            Expression::Boolean(value) => Object::Boolean(*value).into(),
            Expression::String(value) => Object::String(value.clone()).into(),
            Expression::Array(nodes) => eval_array_expression(nodes, environment),
            Expression::Index(index, operands) => eval_index_expression(index.eval(environment)?, operands.eval(environment)?),
            Expression::Prefix(operator, operands) => eval_prefix_expression(&operator, &operands, environment),
            Expression::If(condition, consequence, alternative) => eval_if_expression(condition, consequence, alternative, environment),
        }.map_err(|err| format!("row: {}, col: {}, {err}", self.token.row, self.token.col))
    }
}

fn eval_index_expression(index: Object, operand: Object) -> Result<Object, String> {
    match (index, operand) {
        // (Object::Integer(index), Object::Array(array)) => array.as_ref().get(index).unwrap_or(Err("Array index out of bounds")),
        (Object::Integer(index), Object::Array(array)) => array.get(index as usize)
            .map(|object| object.clone())
            .ok_or(format!("Array index out of bounds index was '{index}' but length was '{}'.", array.len())),
        (index@ _, operand @ _) => Err(format!("Index type '{index}' not allowed on '{operand}'.")),
    }
}

fn eval_s_expression(nodes: &[Node], environment: &mut Environment) -> Result<Object, String> {
    let mut result = Object::Unit;
    for node in nodes {
        result = node.eval(environment)?;
    }
    return result.into();
}


fn eval_let(identifier: &Node, value: &Node, environment: &mut Environment) -> Result<Object, String> {
    let Expression::Identifier(identifier) = &identifier.expression  else {
        return Err(format!("Expected identifier for let-expression at row: {}, col: {}", identifier.token.row, identifier.token.col));
    };

    let value = value.eval(environment)?;

    environment.set(identifier.clone(), value.clone());

    Ok(value)
}

fn eval_if_expression(condition: &Box<Node>, consequence: &Box<Node>, alternative: &Option<Box<Node>>, environment: &mut Environment) -> Result<Object, String> {
    let condition = condition.eval(environment)?;

    return if is_truthy(condition.clone()) {
        consequence.eval(environment)
    } else if let Some(alternative) = alternative {
        alternative.eval(environment)
    } else {
        condition.into()
    }
}

fn eval_identifier(identifier: &Rc<str>, environment: &mut Environment) -> Result<Object, String> {
    if let Some(value) = environment.get(identifier) {
        return Ok(value);
    }
    // if let Some(value) = builtins(identifier.value.as_str()) {
    //     return Ok(value);
    // };
    return Err(format!("No binding for identifier '{}'", identifier));
}

fn eval_array_expression(nodes: &Box<[Node]>, environment: &mut Environment) -> Result<Object, String>{
    let mut objects = Vec::new();
    for node in nodes.iter() {
        objects.push(node.eval(environment)?);
    }

    return Object::Array(Rc::from(objects)).into();
}

fn eval_prefix_expression(operator: &Rc<str>, operands: &[Node], environment: &mut Environment) -> Result<Object, String> {
    match operator.as_ref() {
        "+" => plus_operator(operands, environment),
        "-" => minus_operator(operands, environment),
        "*" => multiply_operator(operands, environment),
        "/" => divide_operator(operands, environment),
        "<" => lesser_then_operator(operands, environment),
        ">" => greater_then_operator(operands, environment),
        "=" => equals_operator(operands, environment),
        "!" => not_operator(operands, environment),
        _ => Err(format!("unknown operator '{operator}'")),
    }
}


// fn plus_operator(left: Object, right: Object) -> Result<Object, String>{
fn plus_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, String>{
    if operands.is_empty() {
        return Object::Integer(0).into();
    }
    if operands.len() == 1 {
        return operands[0].eval(environment);
    }
    operands.iter()
        .map(|node| node.eval(environment))
        .reduce(|left, right| {
            match (left?, right?) {
                (Object::Integer(left), Object::Integer(right)) => {Object::Integer(left + right).into()},

                (Object::Float(left), Object::Integer(right)) => Object::Float(left + f64::from(right)).into(),
                (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) + right).into(),
                (Object::Float(left), Object::Float(right)) => Object::Float(left + right).into(),

                (Object::String(left), right @ _) => Object::String(format!("{left}{}", right.view()).into()).into(),
                (left @ _, Object::String(right)) => Object::String(format!("{}{right}", left.view()).into()).into(),
                (left @ _, right @ _) => Err(format!("Type mismatch (+ {left} {right})")),
            }
        }).unwrap_or(Err("Could not eval operator".to_string()))
}

fn minus_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, String>{
    if operands.is_empty() {
        return Object::Integer(0).into();
    }
    if operands.len() == 1 {
        let object = operands[0].eval(environment)?;
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
            (Object::Integer(left), Object::Integer(right)) => no_truncating_division(left, right),

            (Object::Float(left), Object::Integer(right)) => Object::Float(left / f64::from(right)),
            (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) / right),
            (Object::Float(left), Object::Float(right)) => Object::Float(left / right),
            (left @ _, right @ _)  => return Err(format!("Type mismatch (/ {left} {right})")),
        }
    }

    return result.into();
}

fn no_truncating_division(left: i32, right: i32) -> Object {
    if left % right != 0 {
        Object::Float(f64::from(left) / f64::from(right))
    } else {
        Object::Integer(left / right)
    }
}

fn lesser_then_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, String> {
    if operands.len() <= 1 {
        return Object::Boolean(operands.len() == 1).into()
    }
    // operands.iter()
    //     .map(|node| node.eval(environment))
    //     .reduce(|left, right| {
    //         match (left?, right?) {
    //             (Object::Integer(left), Object::Integer(right)) => Object::Boolean(left < right),
    //             (Object::Float(left), Object::Integer(right)) => Object::Boolean(left < f64::from(right)),
    //             (Object::Integer(left), Object::Float(right)) => Object::Boolean(f64::from(left) < right),
    //             (Object::Float(left), Object::Float(right)) => Object::Boolean(left < right),
    //             (left @ _, right @ _)  => return Err(format!("Type mismatch (< {left} {right})")),
    //         }.into()
    //     }).unwrap_or(Err("Could not eval operator".to_string()))
    // return result.into();
    let mut result = true;
    let mut left = operands[0].eval(environment)?;
    for operand in operands[1..].iter() {
        let right = operand.eval(environment)?;
        result = match (&left, &right) {
            (Object::Integer(left), Object::Integer(right)) => left < right,
            (Object::Float(left), Object::Integer(right)) => *left < f64::from(*right),
            (Object::Integer(left), Object::Float(right)) => f64::from(*left) < *right,
            (Object::Float(left), Object::Float(right)) => left < right,
            (left @ _, right @ _)  => return Err(format!("Type mismatch (< {left} {right})")),
        };
        left = right;
    }
    return Object::Boolean(result).into();
}
fn greater_then_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, String> {
    if operands.len() <= 1 {
        return Object::Boolean(operands.len() == 1).into()
    }
    // operands.iter()
    //     .map(|node| node.eval(environment))
    //     .reduce(|left, right| {
    //         match (left?, right?) {
    //             (Object::Integer(left), Object::Integer(right)) => Object::Boolean(left > right),
    //             (Object::Float(left), Object::Integer(right)) => Object::Boolean(left > f64::from(right)),
    //             (Object::Integer(left), Object::Float(right)) => Object::Boolean(f64::from(left) > right),
    //             (Object::Float(left), Object::Float(right)) => Object::Boolean(left > right),
    //             (left @ _, right @ _)  => return Err(format!("Type mismatch (> {left} {right})")),
    //         }.into()
    //     }).unwrap_or(Err("Could not eval operator".to_string()))
    let mut result = true;
    let mut left = operands[0].eval(environment)?;
    for operand in operands[1..].iter() {
        let right = operand.eval(environment)?;
        result = match (&left, &right) {
            (Object::Integer(left), Object::Integer(right)) => left > right,
            (Object::Float(left), Object::Integer(right)) => *left > f64::from(*right),
            (Object::Integer(left), Object::Float(right)) => f64::from(*left) > *right,
            (Object::Float(left), Object::Float(right)) => left > right,
            (left @ _, right @ _)  => return Err(format!("Type mismatch (> {left} {right})")),
        };
        left = right;
    }
    return Object::Boolean(result).into();
}

fn equals_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, String> {
    if operands.len() <= 1 {
        return Object::Boolean(operands.len() == 1).into()
    }

    let mut result = true;
    let mut left = operands[0].eval(environment)?;
    for operand in operands[1..].iter() {
        let right = operand.eval(environment)?;
        result = match (&left, &right) {
            (Object::Integer(left), Object::Integer(right)) => left == right,
            (Object::Float(left), Object::Integer(right)) => *left == f64::from(*right),
            (Object::Integer(left), Object::Float(right)) => f64::from(*left) == *right,
            (Object::Float(left), Object::Float(right)) => left == right,
            (left @ _, right @ _)  => return Err(format!("Type mismatch (= {left} {right})")),
        };
        left = right;
    }
    return Object::Boolean(result).into();
}

fn not_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, String> {
    // match (operands.get(0), ) {
    //     None => Object::Boolean(true).into(),
    //     Some(node) => Object::Boolean(!is_truthy(node.eval(environment)?)).into(),
    //     _ =>  todo!(),
    // }
    if operands.len() > 1 {
        return Err(format!( "Operator ! expects only 1 operand got {}", operands.len()));
    }

    if let Some(node) = operands.get(0) {
        Object::Boolean(!is_truthy(node.eval(environment)?))
    } else { Object::Boolean(true) }.into()
}

fn is_truthy(object: Object) -> bool {
    match object {
        Object::Boolean(true) => true,
        Object::Boolean(false)
        | Object::Integer(0) => false,
        Object::Float(value) => value != 0.0,
        _ => true,
    }
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