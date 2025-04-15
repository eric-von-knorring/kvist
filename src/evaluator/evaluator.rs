use std::collections::VecDeque;
use std::rc::Rc;

use crate::ast::ast::{Node, Program};
use crate::ast::expression::Expression;
use crate::evaluator::builtin::builtins;
use crate::evaluator::error::{EvaluationError, ToEvaluationError};
use crate::evaluator::include::include_script;
use crate::evaluator::operator_expression::eval_operator_expression;
use crate::object::environment::Environment;
use crate::object::object::Object;

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
            Expression::Operator(operator, operands) => eval_operator_expression(&operator, &operands, environment),
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


    return if condition.is_truthy() {
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

        if condition_result.is_truthy() {
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
        if !&condition.is_truthy() {
            return condition.into();
        }
    }
}

fn eval_while_body_expression(condition: &Box<Node>, loop_body: &Box<Node>, environment: &mut Environment) -> Result<Object, EvaluationError> {
    loop {
        let condition = condition.eval(environment)?
            .spread_to_single()
            .unwrap_or(Object::Unit);
        if !&condition.is_truthy() {
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