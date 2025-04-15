use crate::ast::ast::Node;
use crate::evaluator::error::EvaluationError;
use crate::evaluator::queued_evaluator::QueuedEvaluator;
use crate::object::environment::Environment;
use crate::object::object::{Object, Viewable};
use std::rc::Rc;

pub(crate) fn eval_operator_expression(operator: &Rc<str>, operands: &[Node], environment: &mut Environment) -> Result<Object, EvaluationError> {
    let queued_evaluator = QueuedEvaluator::new(operands, environment);
    match operator.as_ref() {
        "+" => plus_operator(queued_evaluator),
        "-" => minus_operator(queued_evaluator),
        "*" => multiply_operator(queued_evaluator),
        "/" => divide_operator(queued_evaluator),
        "<" => lesser_than_operator(queued_evaluator),
        ">" => greater_than_operator(queued_evaluator),
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

fn lesser_than_operator(mut queued_evaluator: QueuedEvaluator) -> Result<Object, EvaluationError> {
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

fn greater_than_operator(mut queued_evaluator: QueuedEvaluator) -> Result<Object, EvaluationError> {
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
        Object::Boolean(!&object?.is_truthy())
    } else { Object::Boolean(true) }.into()
}
