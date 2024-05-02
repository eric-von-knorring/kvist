use std::env;

mod token;
mod lexer;
mod parser;
mod ast;
mod evaluator;
mod object;
mod repl;
mod script;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        repl::repl::start();
    } else {
        script::script::start(&args[1]);
    }
}
