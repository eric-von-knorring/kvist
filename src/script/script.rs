

use std::fs::read_to_string;
use crate::evaluator::evaluator::Eval;
use crate::lexer::lexer::Lexer;
use crate::object::environment::Environment;
use crate::parser::parser::Parser;

pub(crate) fn start(file: &String) {
    let mut env = Environment::new();

    let Ok(content) = read_to_string(file) else {
        eprintln!("Could not open: {file}");
        return;
    };

    let lexer = Lexer::from(content.as_str());
    let parser = Parser::from(lexer);

    let evaluation = match parser.parse_program() {
        Ok(result) => result.eval(&mut env),
        Err(errors) => {
            print_errors(errors);
            return;
        }
    };

    match evaluation {
        Ok(_) => {},
        Err(error) => eprintln!("Execution error:\n\tERROR: {error}")
    };
}

fn print_errors(errors: Vec<String>) {
    eprintln!("Failed to parse input:");
    errors.iter()
        .for_each(|error| eprintln!("\tERROR: {error}"))
}
