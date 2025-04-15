use std::fmt::{Display, Formatter};
use crate::ast::ast::Node;
use crate::object::object::Object;

#[derive(Debug)]
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
        EvaluationError::Simple(value)
    }
}
impl From<ContextualEvaluationError> for EvaluationError {
    fn from(value: ContextualEvaluationError) -> Self {
        EvaluationError::Contextual(value)
    }
}

impl Display for EvaluationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::Simple(message) => write!(f, "{}", message),
            EvaluationError::Contextual(error) => write!(f, "Row {}, Col: {}: {}", error.row, error.col, error.message),
        }
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
