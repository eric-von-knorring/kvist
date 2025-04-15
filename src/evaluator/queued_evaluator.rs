use std::collections::VecDeque;
use crate::ast::ast::Node;
use crate::evaluator::error::{EvaluationError, ToEvaluationError};
use crate::evaluator::evaluator::Eval;
use crate::object::environment::Environment;
use crate::object::object::Object;

pub struct QueuedEvaluator<'a> {
    buf: VecDeque<Object>,
    index: usize,
    nodes: &'a[Node],
    environment: &'a mut Environment,
}

impl QueuedEvaluator<'_> {

    pub fn new<'a>(nodes: &'a[Node], environment: &'a mut Environment) -> QueuedEvaluator<'a> {
        QueuedEvaluator {
            buf: Default::default(),
            index: 0,
            nodes,
            environment,
        }
    }

    pub fn next(&mut self) -> Option<Result<Object, EvaluationError>> {
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

    pub fn has_next(&self) -> bool {
        !self.buf.is_empty() || self.index < self.nodes.len()
    }

    pub fn queue_len(&self) -> usize {
        self.buf.len() + self.nodes.len() - self.index
    }
}
