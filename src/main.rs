
mod token;
mod lexer;
mod parser;
mod ast;
mod evaluator;
mod object;
mod repl;

fn main() {
    repl::repl::start();
}
