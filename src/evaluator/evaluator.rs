use std::collections::VecDeque;
use std::rc::Rc;

use crate::ast::ast::{Node, Program};
use crate::ast::expression::Expression;
use crate::evaluator::builtin::builtins;
use crate::evaluator::error::{ContextualEvaluationError, EvaluationError};
use crate::evaluator::include::include_script;
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
            Expression::Function(params, vararg, body) => Object::Function(params.clone(), vararg.clone(), body.clone(), environment.clone().into()).into(),
            Expression::Section(section) => eval_scope_section(section, environment),
            Expression::Include(target) => eval_include_expression(target, environment),
            Expression::Spread(operand) => eval_spread_expression(operand.eval(environment)?),
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

fn eval_spread_expression(operand: Object) -> Result<Object, EvaluationError> {
    match operand {
        Object::Array(array) => Object::Spread(array).into(),
        operand @ _ => Err(format!("Spread-operator not allowed on '{operand}'.").into()),
    }
}

fn eval_expression_literal(nodes: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    let Some(node) = nodes.get(0) else {
        return Object::Unit.into();
    };

    match node.eval(environment) {
        Ok(Object::Function(params, vararg, body, env)) => {
            eval_function_call(node, params, vararg, nodes, body, &mut Environment::from(env), environment)
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
            } else {
                result?.spread_to_single()
                    .unwrap_or(Object::Unit).into()
            }
        }
        err @ Err(_) => err,
    }
}

fn eval_function_call(node: &Node, params: Rc<[Node]>, vararg: Rc<Option<Node>>, nodes: &[Node], body: Rc<Node>, function_environment: &mut Environment, environment: &mut Environment) -> Result<Object, EvaluationError> {
    let mut args_queue = VecDeque::new();
    while args_queue.len() < params.len() {
        if let Some(node) = nodes.get(args_queue.len() + 1) {
            node.eval(environment)?
                .expand_spread(|object| args_queue.push_back(object));
        } else { break; }
    }

    for  param in params.iter() {
        let Expression::Identifier(ref name) = param.expression else {
            return Err(node.to_error(format!("Illegal function parameter type {param:?}")));
        };

        let value = args_queue.pop_front()
            .ok_or(node.to_error(format!("Missing parameter value for {name}")))?;
        function_environment.set(name.clone(), value)
    }

    if let Some(vararg_name) = vararg.as_ref() {
        let Expression::Identifier(ref name) = vararg_name.expression else {
            return Err(node.to_error(format!("Illegal function parameter type {vararg_name:?}")));
        };
        let mut args = Vec::new();

        while let Some(arg) = args_queue.pop_front() {
            args.push(arg);
        }

        for node in nodes[params.len()+1..].iter() {
            node.eval(environment)?
                .expand_spread(|object| args.push(object));
        }
        function_environment.set(name.clone(), Object::Array(args.into()))
    }

    body.eval(function_environment)
}

fn eval_builtin(builtin: fn(Box<[Object]>) -> Result<Object, String>, args: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    let mut param = Vec::new();
    for arg in args {
        arg.eval(environment)?.expand_spread(|object| param.push(object));
    }
    match builtin(param.into()) {
        Ok(result) => result.into(),
        Err(message) => EvaluationError::Simple(message).into(),
    }
}

fn eval_expression_nodes(nodes: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    let mut result = Object::Unit;
    for node in nodes {
        result = node.eval(environment)?
            .spread_to_single()
            .unwrap_or(result);
    }
    return result.into();
}

fn eval_scope_section(node: &Box<Node>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    node.eval(&mut Environment::from(Rc::from(environment.clone())))
}

fn eval_include_expression(target: &Box<Node>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    match target.eval(environment)? {
        Object::String(target_path) => include_script(target_path.as_ref(), environment),
        _ => target.to_error("Illegal include expression. Expected target to be a string.".to_owned()).into(),
    }
}


fn eval_set(variables: &Rc<[(Node, Node)]>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    let mut result = Object::Unit;
    for (identifier, value) in variables.iter() {
        let Expression::Identifier(identifier) = &identifier.expression  else {
            return Err(identifier.to_error("Expected identifier for set-expression".to_string()));
        };

        result = value.eval(environment)?
            .spread_to_single()
            .unwrap_or(Object::Unit);

        environment.set(identifier.clone(), result.clone());
    }

    Ok(result)
}

fn eval_if_expression(condition: &Box<Node>,
                      consequence: &Box<Node>,
                      alternative: &Option<Box<Node>>,
                      environment: &mut Environment) -> Result<Object, EvaluationError> {
    let condition = condition.eval(environment)?
        .spread_to_single()
        .unwrap_or(Object::Unit);


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
        condition_result = condition.eval(environment)?
            .spread_to_single()
            .unwrap_or(Object::Unit);

        if is_truthy(&condition_result) {
            return consequence.eval(environment);
        }
    };
    return condition_result.into();
}

fn eval_while_expression(condition: &Box<Node>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    loop {
        let condition = condition.eval(environment)?
            .spread_to_single()
            .unwrap_or(Object::Unit);
        if !is_truthy(&condition) {
            return condition.into();
        }
    }
}

fn eval_while_body_expression(condition: &Box<Node>, loop_body: &Box<Node>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    loop {
        let condition = condition.eval(environment)?
            .spread_to_single()
            .unwrap_or(Object::Unit);
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
        node.eval(environment)?.expand_spread(|object| objects.push(object));
    }

    return Object::Array(Rc::from(objects)).into();
}

struct QueuedEvaluator<'a> {
    buf: VecDeque<Object>,
    index: usize,
    nodes: &'a[Node],
    environment: &'a mut Environment,
}

impl QueuedEvaluator<'_> {

    fn new<'a>(nodes: &'a[Node], environment: &'a mut Environment) -> QueuedEvaluator<'a> {
        QueuedEvaluator {
            buf: Default::default(),
            index: 0,
            nodes,
            environment,
        }
    }

    fn next(&mut self) -> Option<Result<Object, EvaluationError>> {
        if let Some(object) = self.buf.pop_front() {
            return Result::from(object).into();
        }
        let Some(node) = self.nodes.get(self.index) else {
            return None;
        };
        let result = match node.eval(self.environment) {
            Ok(Object::Spread(objects)) => {
                for object in &objects[1..] {
                    self.buf.push_back(object.clone());
                }
                objects.get(0).map(|o| o.clone())
                    .ok_or(node.to_error("Missing argument for spread operator".to_string()))
            }
            result @ _ => result,
        };
        self.index += 1;
        result.into()
    }

    fn has_next(&self) -> bool {
        !self.buf.is_empty() || self.index < self.nodes.len()
    }

    fn queue_len(&self) -> usize {
        self.buf.len() + self.nodes.len() - self.index
    }
}

fn eval_prefix_expression(operator: &Rc<str>, operands: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    let mut operand_objects = VecDeque::new();
    for operand in operands {
        operand.eval(environment)?.expand_spread(|operand| operand_objects.push_back(operand));
    }
    let queued_evaluator = QueuedEvaluator::new(operands, environment);
    match operator.as_ref() {
        "+" => plus_operator(queued_evaluator),
        "-" => minus_operator(queued_evaluator),
        "*" => multiply_operator(queued_evaluator),
        "/" => divide_operator(queued_evaluator),
        "<" => lesser_then_operator(queued_evaluator),
        ">" => greater_then_operator(queued_evaluator),
        "=" => equals_operator(queued_evaluator),
        "!" => not_operator(queued_evaluator),
        _ => Err(format!("unknown operator '{operator}'").into()),
    }
}


fn plus_operator(mut queued_evaluator: QueuedEvaluator) -> Result<Object, EvaluationError> {
    let Some(first) = queued_evaluator.next() else {
        return Object::Integer(0).into();
    };
    if !queued_evaluator.has_next() {
        return first;
    }
    let mut left = first?;

    // while let Some(right) = operands.pop_front() {
    while let Some(right) = queued_evaluator.next() {
        left = match (left, right?) {
            (Object::Integer(left), Object::Integer(right)) => { Object::Integer(left + right).into() }
            (Object::Float(left), Object::Integer(right)) => Object::Float(left + f64::from(right)).into(),
            (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) + right).into(),
            (Object::Float(left), Object::Float(right)) => Object::Float(left + right).into(),

            (Object::String(left), right @ _) => Object::String(format!("{left}{}", right.view()).into()).into(),
            (left @ _, Object::String(right)) => Object::String(format!("{}{right}", left.view()).into()).into(),
            (left @ _, right @ _) => return EvaluationError::from(format!("Type mismatch (+ {left} {right})").to_string()).into(),
        }
    }
    Ok(left)
}

fn minus_operator(mut queued_evaluator: QueuedEvaluator) -> Result<Object, EvaluationError> {
    let Some(first) = queued_evaluator.next() else {
        return Object::Integer(0).into();
    };
    if !queued_evaluator.has_next() {
        return match first? {
            Object::Integer(value) => Object::Integer(0 - value).into(),
            Object::Float(value) => Object::Float(0. - value).into(),
            object @ _ => EvaluationError::from(format!("Type mismatch (- {object})")).into(),
        };
    }
    let mut left = first?;

    while let Some(right) = queued_evaluator.next() {
        left = match (left, right?) {
            (Object::Integer(left), Object::Integer(right)) => Object::Integer(left - right).into(),

            (Object::Float(left), Object::Integer(right)) => Object::Float(left - f64::from(right)).into(),
            (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) - right).into(),
            (Object::Float(left), Object::Float(right)) => Object::Float(left - right).into(),

            (left @ _, right @ _) => return EvaluationError::from(format!("Type mismatch (+ {left} {right})").to_string()).into(),
        }
    }
    Ok(left)
}

fn multiply_operator(mut queued_evaluator: QueuedEvaluator) -> Result<Object, EvaluationError> {
    let mut product = Object::Integer(1);
    while let Some(operand) = queued_evaluator.next() {
        product = match (product, operand?) {
            (Object::Integer(left), Object::Integer(right)) => Object::Integer(left * right),
            (Object::Float(left), Object::Integer(right)) => Object::Float(left * f64::from(right)),
            (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) * right),
            (Object::Float(left), Object::Float(right)) => Object::Float(left * right),
            (left @ _, right @ _) => return EvaluationError::from(format!("Type mismatch (* {left} {right})").to_string()).into(),
        };
    }
    product.into()
}

fn divide_operator(mut queued_evaluator: QueuedEvaluator) -> Result<Object, EvaluationError> {
    let Some(first) = queued_evaluator.next() else {
        return Object::Undefined.into();
    };
    if !queued_evaluator.has_next() {
        return match first? {
            Object::Integer(0)   => Object::Undefined.into(),
            Object::Float(value) if value == 0. => Object::Undefined.into(),
            Object::Integer(value) => Object::Float(1. / f64::from(value)).into(),
            Object::Float(value) => Object::Float(1. / value).into(),
            object @ _ => EvaluationError::from(format!("Type mismatch (/ {object})").to_string()).into(),
        };
    }
    let mut result = first?;

    // for operand in operands {
    while let Some(operand) = queued_evaluator.next() {
        result = match (result, operand?) {
            (_, Object::Integer(0)) => Object::Undefined.into(),
            (_, Object::Float(value)) if value == 0. => Object::Undefined.into(),
            (Object::Integer(left), Object::Integer(right)) => no_truncating_division(left, right),
            (Object::Float(left), Object::Integer(right)) => Object::Float(left / f64::from(right)),
            (Object::Integer(left), Object::Float(right)) => Object::Float(f64::from(left) / right),
            (Object::Float(left), Object::Float(right)) => Object::Float(left / right),
            (left @ _, right @ _) => return Err(format!("Type mismatch (/ {left} {right})").into()),
        }
    }

    result.into()
}

fn no_truncating_division(left: i32, right: i32) -> Object {
    if left % right != 0 {
        Object::Float(f64::from(left) / f64::from(right))
    } else {
        Object::Integer(left / right)
    }
}

fn lesser_then_operator(mut queued_evaluator: QueuedEvaluator) -> Result<Object, EvaluationError> {
    let Some(first) = queued_evaluator.next() else {
        return Object::Boolean(false).into();
    };
    if !queued_evaluator.has_next() {
        return Object::Boolean(true).into();
    }
    let mut left = first?;
    let mut result = true;

    while let Some(operand) = queued_evaluator.next() {
        let right = operand?;
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
    Object::Boolean(result).into()
}

fn greater_then_operator(mut queued_evaluator: QueuedEvaluator) -> Result<Object, EvaluationError> {
    let Some(first) = queued_evaluator.next() else {
        return Object::Boolean(false).into();
    };
    if !queued_evaluator.has_next() {
        return Object::Boolean(true).into();
    }
    let mut left = first?;
    let mut result = true;

    while let Some(operand) = queued_evaluator.next() {
        let right = operand?;
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
    Object::Boolean(result).into()
}

fn equals_operator(mut queued_evaluator: QueuedEvaluator) -> Result<Object, EvaluationError> {

    let Some(first) = queued_evaluator.next() else {
        return Object::Boolean(false).into();
    };
    if !queued_evaluator.has_next() {
        return Object::Boolean(true).into();
    }
    let mut left = first?;
    let mut result = true;

    // for operand in operands {
    while let Some(operand) = queued_evaluator.next() {
        let right = operand?;
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
    Object::Boolean(result).into()
}

fn not_operator(mut queued_evaluator: QueuedEvaluator) -> Result<Object, EvaluationError> {
    if let Some(object) = queued_evaluator.next() {
        if queued_evaluator.has_next() {
            return Err(format!("Operator ! expects only 1 operand found at least {}", queued_evaluator.queue_len() + 1).into());
        }
        Object::Boolean(!is_truthy(&object?))
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
