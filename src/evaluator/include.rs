use std::fs::read_to_string;
use crate::evaluator::error::EvaluationError;
use crate::evaluator::evaluator::Eval;
use crate::lexer::lexer::Lexer;
use crate::object::environment::Environment;
use crate::object::object::Object;
use crate::parser::parser::Parser;


pub fn include_script(target: &str, environment: &mut Environment) -> Result<Object, EvaluationError> {

    let Ok(content) = read_to_string(target) else {
        return EvaluationError::Simple(format!("Include error. Could not open: {target}")).into();
    };

    let lexer = Lexer::from(content.as_str());
    let parser = Parser::from(lexer);

     let evaluation = match parser.parse_program() {
        Ok(result) => result.eval(environment),
        Err(errors) => {
            eprintln!("Could not include file '{target}'. Parse error:");
            errors.iter()
                .for_each(|error| eprintln!("\tERROR: {error}"));
            return EvaluationError::Simple(format!("Could not include file '{target}'")).into()
        }
    };

    match evaluation {
        ok @ Ok(_) => ok,
        Err(error) => {
            eprintln!("Could not include file '{target}'. Execution error:\n\tERROR: {error}");
            EvaluationError::Simple(format!("Could not include file '{target}'")).into()
        }
    }
}