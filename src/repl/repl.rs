use std::io;
use std::io::{BufRead, BufReader, Write};

use crate::evaluator::evaluator::Eval;
use crate::lexer::lexer::Lexer;
use crate::object::environment::Environment;
use crate::object::object::Viewable;
use crate::parser::parser::Parser;

// const PROMPT: &str = ">> ";

pub fn start() {
    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut env = Environment::new();
    println!("Feel free to type in commands");
    loop {
        print!(">> ");
        stdout.flush().unwrap();


        let mut line: String = String::new();
        stdin.read_line(&mut line).expect("Failed to read line");
        if line == "\n" {
            continue;
        }

        let lexer = Lexer::from(line.as_str());
        let parser = Parser::from(lexer);
        // match parser.parse_program().map(|program| program.eval(&mut env)) {
        let evaluation = match parser.parse_program() {
            Ok(result) => result.eval(&mut env),
            Err(errors) => {
                print_errors(errors);
                continue;
            }
        };
        match evaluation {
            Ok(object) => println!("{}", object.view()),
            Err(error) => eprintln!("Execution error:\n\tERROR: {error}")
        };
    }
}

fn print_errors(errors: Vec<String>) {
    eprintln!("Failed to parse input:");
    errors.iter()
        .for_each(|error| eprintln!("\tERROR: {error}"))
}