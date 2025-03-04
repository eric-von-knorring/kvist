use std::rc::Rc;

use crate::ast::ast::{Node, Program};
use crate::ast::expression::Expression;
use crate::evaluator::builtin::builtins;
use crate::evaluator::error::{ContextualEvaluationError, EvaluationError};
use crate::object::environment::Environment;
use crate::object::object::{Object, Viewable};

pub trait Eval {
    fn eval(&self, environment: &mut Environment) -> Result<Object, EvaluationError>;
}

impl Eval for Program {
    // fn eval(&self, environment: &mut Environment) -> Result<Object, String> {
    fn eval(&self, environment: &mut Environment) -> Result<Object, EvaluationError> {
        let mut result = Object::Integer(0);
        for node in self.nodes.iter() {
            result = node.eval(environment)?;
        }
        result.into()
    }
}

impl Eval for Node {
    fn eval(&self, environment: &mut Environment) -> Result<Object, EvaluationError> {
        match &self.expression {
            Expression::ExpressionLiteral(nodes) => eval_expression_literal(nodes, environment),
            Expression::Set(variables) => eval_set(variables, environment),
            Expression::Identifier(value) => eval_identifier(value, environment),
            Expression::Integer(value) => Object::Integer(*value).into(),
            Expression::Float(value) => Object::Float(*value).into(),
            Expression::Boolean(value) => Object::Boolean(*value).into(),
            Expression::String(value) => Object::String(value.clone()).into(),
            Expression::Array(nodes) => eval_array_expression(nodes, environment),
            Expression::Index(index, operands) => eval_index_expression(index.eval(environment)?, operands.eval(environment)?),
            Expression::Prefix(operator, operands) => eval_prefix_expression(&operator, &operands, environment),
            Expression::If(condition, consequence, alternative) => eval_if_expression(condition, consequence, alternative, environment),
            Expression::When(branches) => eval_when_expression(branches, environment),
            Expression::While(condition, None) => eval_while_expression(condition, environment),
            Expression::While(condition, Some(loop_body)) => eval_while_body_expression(condition, loop_body, environment),
            Expression::Function(params, body) => Object::Function(params.clone(), body.clone(), environment.clone().into()).into(),
            Expression::Section(section) => eval_scope_section(section, environment),
        }.map_err(|err| match err {
            EvaluationError::Simple(message) => self.to_error(message),
            err @ _ => err,
        })
    }
}

pub trait ToEvaluationError {
    fn to_error(&self, message: String) -> EvaluationError;
}

impl ToEvaluationError for Node {
    fn to_error(&self, message: String) -> EvaluationError {
        ContextualEvaluationError {
            col: self.token.col,
            row: self.token.row,
            message,
        }.into()
    }
}

impl From<EvaluationError> for Result<Object, EvaluationError> {
    fn from(value: EvaluationError) -> Self {
        Err(value)
    }
}

fn eval_index_expression(index: Object, operand: Object) -> Result<Object, EvaluationError> {
    match (index, operand) {
        // (Object::Integer(index), Object::Array(array)) => array.as_ref().get(index).unwrap_or(Err("Array index out of bounds")),
        (Object::Integer(index), Object::Array(array)) => array.get(index as usize)
            .map(|object| object.clone())
            .ok_or(format!("Array index out of bounds index was '{index}' but length was '{}'.", array.len()).into()),
        (index @ _, operand @ _) => Err(format!("Index type '{index}' not allowed on '{operand}'.").into()),
    }
}

fn eval_expression_literal(nodes: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    let Some(node) = nodes.get(0) else {
        return Object::Unit.into();
    };

    match node.eval(environment) {
        Ok(Object::Function(params, body, env)) => {
            eval_function_call(node, params, nodes, body, &mut Environment::from(env), environment)
        }
        Ok(Object::Builtin(builtin)) => {
            eval_builtin(builtin, &nodes[1..], environment)
                .map_err(|err| match err {
                    EvaluationError::Simple(message) => node.to_error(message),
                    err @ _ => err,
                })
        }
        result @ Ok(_) => {
            if nodes.len() > 1 {
                eval_expression_nodes(&nodes[1..], environment)
            } else { result }
        }
        err @ Err(_) => err,
    }
}

fn eval_function_call(node: &Node, params: Rc<[Node]>, nodes: &[Node], body: Rc<Node>, function_environment: &mut Environment, environment: &mut Environment) -> Result<Object, EvaluationError> {
    for (index, param) in params.iter().enumerate() {
        let Expression::Identifier(ref name) = param.expression else {
            return Err(node.to_error(format!("Illegal function primate {param:?}")));
        };

        let value = nodes.get(index + 1)
            .ok_or(node.to_error(format!("Missing parameter value for {name}")))?
            .eval(environment)?;
        function_environment.set(name.clone(), value)
    }

    body.eval(function_environment)
}

fn eval_builtin(builtin: fn(Box<[Object]>) -> Result<Object, String>, args: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    let mut param = Vec::new();
    for arg in args {
        param.push(arg.eval(environment)?)
    }
    match builtin(param.into()) {
        Ok(result) => result.into(),
        Err(message) => EvaluationError::Simple(message).into(),
    }
}

fn eval_expression_nodes(nodes: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    let mut result = Object::Unit;
    for node in nodes {
        result = node.eval(environment)?;
    }
    return result.into();
}

fn eval_scope_section(node: &Box<Node>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    node.eval(&mut Environment::from(Rc::from(environment.clone())))
}


fn eval_set(variables: &Rc<[(Node, Node)]>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    let mut result = Object::Unit;
    for (identifier, value) in variables.iter() {
        let Expression::Identifier(identifier) = &identifier.expression  else {
            return Err(identifier.to_error("Expected identifier for set-expression".to_string()));
        };

        result = value.eval(environment)?;

        environment.set(identifier.clone(), result.clone());
    }

    Ok(result)
}

fn eval_if_expression(condition: &Box<Node>,
                      consequence: &Box<Node>,
                      alternative: &Option<Box<Node>>,
                      environment: &mut Environment) -> Result<Object, EvaluationError> {
    let condition = condition.eval(environment)?;

    return if is_truthy(&condition) {
        consequence.eval(environment)
    } else if let Some(alternative) = alternative {
        alternative.eval(environment)
    } else {
        condition.into()
    };
}

fn eval_when_expression(branches: &Box<[(Box<Node>, Box<Node>)]>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    let mut condition_result = Object::Unit;

    for (condition, consequence) in branches.iter() {
        condition_result = condition.eval(environment)?;
        if is_truthy(&condition_result) {
            return consequence.eval(environment);
        }
    };
    return condition_result.into();
}

fn eval_while_expression(condition: &Box<Node>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    loop {
        let condition = condition.eval(environment)?;
        if !is_truthy(&condition) {
            return condition.into();
        }
    }
}

fn eval_while_body_expression(condition: &Box<Node>, loop_body: &Box<Node>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    loop {
        let condition = condition.eval(environment)?;
        if !is_truthy(&condition) {
            return condition.into();
        }
        loop_body.eval(environment)?;
    }
}


fn eval_identifier(identifier: &Rc<str>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    if let Some(value) = environment.get(identifier) {
        return Ok(value);
    }
    if let Some(value) = builtins(identifier.as_ref()) {
        return Ok(value);
    };
    return Err(format!("No binding for identifier '{}'", identifier).into());
}

fn eval_array_expression(nodes: &Box<[Node]>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    let mut objects = Vec::new();
    for node in nodes.iter() {
        objects.push(node.eval(environment)?);
    }

    return Object::Array(Rc::from(objects)).into();
}

fn eval_prefix_expression(operator: &Rc<str>, operands: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    match operator.as_ref() {
        "+" => plus_operator(operands, environment),
        "-" => minus_operator(operands, environment),
        "*" => multiply_operator(operands, environment),
        "/" => divide_operator(operands, environment),
        "<" => lesser_then_operator(operands, environment),
        ">" => greater_then_operator(operands, environment),
        "=" => equals_operator(operands, environment),
        "!" => not_operator(operands, environment),
        _ => Err(format!("unknown operator '{operator}'").into()),
    }
}


// fn plus_operator(left: Object, right: Object) -> Result<Object, String>{
fn plus_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
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
                (Object::Integer(left), Object::Integer(right)) => { Object::Integer(left + right).into() }

                (Object::Float(left), Object::Integer(right)) => Object::Float(left + f64::from(right)).into(),
                (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) + right).into(),
                (Object::Float(left), Object::Float(right)) => Object::Float(left + right).into(),

                (Object::String(left), right @ _) => Object::String(format!("{left}{}", right.view()).into()).into(),
                (left @ _, Object::String(right)) => Object::String(format!("{}{right}", left.view()).into()).into(),
                (left @ _, right @ _) => Err(format!("Type mismatch (+ {left} {right})").into()),
            }
        }).unwrap_or(Err("Could not eval operator".to_string().into()))
}

fn minus_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    if operands.is_empty() {
        return Object::Integer(0).into();
    }
    if operands.len() == 1 {
        let object = operands[0].eval(environment)?;
        return match operands[0].eval(environment)? {
            Object::Integer(value) => Object::Integer(0 - value).into(),
            Object::Float(value) => Object::Float(0. - value).into(),
            _ => Err(operands[0].to_error(format!("Type mismatch (- {object})"))),
        };
    }
    operands.iter()
        .map(|node| node.eval(environment))
        .reduce(|left, right| {
            match (left?, right?) {
                (Object::Integer(left), Object::Integer(right)) => Object::Integer(left - right).into(),

                (Object::Float(left), Object::Integer(right)) => Object::Float(left - f64::from(right)).into(),
                (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) - right).into(),
                (Object::Float(left), Object::Float(right)) => Object::Float(left - right).into(),

                (left @ _, right @ _) => Err(format!("Type mismatch (- {left} {right})").into()),
            }
        }).unwrap_or(Err("Could not eval operator".to_string().into()))
}

fn multiply_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    let mut product = Object::Integer(1);
    for operand in operands {
        product = match (product, operand.eval(environment)?) {
            (Object::Integer(left), Object::Integer(right)) => Object::Integer(left * right),
            (Object::Float(left), Object::Integer(right)) => Object::Float(left * f64::from(right)),
            (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) * right),
            (Object::Float(left), Object::Float(right)) => Object::Float(left * right),
            (left @ _, right @ _) => return Err(operand.to_error(format!("Type mismatch (* {left} {right})"))),
        };
    }

    return product.into();
}

fn divide_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    if operands.len() == 0 {
        return Object::Undefined.into();
    }
    if let Some(single) = operands.single() {
        return match single.eval(environment)? {
            Object::Integer(0)   => Object::Undefined.into(),
            Object::Float(value) if value == 0. => Object::Undefined.into(),
            Object::Integer(value) => Object::Float(1. / f64::from(value)).into(),
            Object::Float(value) => Object::Float(1. / value).into(),
            object @ _ => single.to_error(format!("Type mismatch (/ {object})")).into()
        };
    }

    let mut result = operands[0].eval(environment)?;
    for operand in operands[1..].iter() {
        result = match (result, operand.eval(environment)?) {
            (_, Object::Integer(0)) => Object::Undefined.into(),
            (_, Object::Float(value)) if value == 0. => Object::Undefined.into(),
            (Object::Integer(left), Object::Integer(right)) => no_truncating_division(left, right),
            (Object::Float(left), Object::Integer(right)) => Object::Float(left / f64::from(right)),
            (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) / right),
            (Object::Float(left), Object::Float(right)) => Object::Float(left / right),
            (left @ _, right @ _) => return Err(format!("Type mismatch (/ {left} {right})").into()),
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

fn lesser_then_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    if operands.len() <= 1 {
        return Object::Boolean(operands.len() == 1).into();
    }
    let mut result = true;
    let mut left = operands[0].eval(environment)?;
    for operand in operands[1..].iter() {
        let right = operand.eval(environment)?;
        result = match (&left, &right) {
            (Object::Integer(left), Object::Integer(right)) => left < right,
            (Object::Float(left), Object::Integer(right)) => *left < f64::from(*right),
            (Object::Integer(left), Object::Float(right)) => f64::from(*left) < *right,
            (Object::Float(left), Object::Float(right)) => left < right,
            (left @ _, right @ _) => return Err(format!("Type mismatch (< {left} {right})").into()),
        };
        if !result {
            return Object::Boolean(false).into();
        }
        left = right;
    }
    return Object::Boolean(result).into();
}

fn greater_then_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    if operands.len() <= 1 {
        return Object::Boolean(operands.len() == 1).into();
    }
    let mut result = true;
    let mut left = operands[0].eval(environment)?;
    for operand in operands[1..].iter() {
        let right = operand.eval(environment)?;
        result = match (&left, &right) {
            (Object::Integer(left), Object::Integer(right)) => left > right,
            (Object::Float(left), Object::Integer(right)) => *left > f64::from(*right),
            (Object::Integer(left), Object::Float(right)) => f64::from(*left) > *right,
            (Object::Float(left), Object::Float(right)) => left > right,
            (left @ _, right @ _) => return Err(format!("Type mismatch (> {left} {right})").into()),
        };
        if !result {
            return Object::Boolean(false).into();
        }
        left = right;
    }
    return Object::Boolean(result).into();
}

fn equals_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    if operands.len() <= 1 {
        return Object::Boolean(operands.len() == 1).into();
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
            (Object::String(left), Object::String(right)) => left == right,
            (left @ _, right @ _) => return Err(format!("Type mismatch (= {left} {right})").into()),
        };
        if !result {
            return Object::Boolean(false).into();
        }
        left = right;
    }
    return Object::Boolean(result).into();
}

fn not_operator(operands: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    if operands.len() > 1 {
        return Err(format!("Operator ! expects only 1 operand got {}", operands.len()).into());
    }

    if let Some(node) = operands.get(0) {
        Object::Boolean(!is_truthy(&node.eval(environment)?))
    } else { Object::Boolean(true) }.into()
}

fn is_truthy(object: &Object) -> bool {
    match object {
        Object::Boolean(true) => true,
        Object::Boolean(false)
        | Object::Integer(0) => false,
        Object::Float(value) => *value != 0.0,
        _ => true,
    }
}

impl<T> From<Object> for Result<Object, T> {
    fn from(value: Object) -> Self {
        Ok(value)
    }
}

trait Single<T> {
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