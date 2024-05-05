use std::fmt::{Display, Formatter};
use crate::evaluator::error::EvaluationError::{Contextual, Simple};


pub enum EvaluationError {
    Simple(String),
    Contextual(ContextualEvaluationError),
}

#[derive(Debug)]
pub struct ContextualEvaluationError {
    pub col: u32,
    pub row: u32,
    pub message: String,
}

impl From<String> for EvaluationError {
    fn from(value: String) -> Self {
        Simple(value)
    }
}
impl From<ContextualEvaluationError> for EvaluationError {
    fn from(value: ContextualEvaluationError) -> Self {
        Contextual(value)
    }
}

impl Display for EvaluationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Simple(message) => write!(f, "{}", message),
            Contextual(error) => write!(f, "Row {}, Col: {}: {}", error.row, error.col, error.message)
        }
    }
}
