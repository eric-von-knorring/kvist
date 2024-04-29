use std::{env, io};
use std::io::{BufRead, BufReader};
use std::rc::Rc;
use crate::object::object::{Object, Viewable};

pub fn builtins(name: &str) -> Option<Object> {
    match name {
        "args" => Some(Object::Builtin(args)),
        "println" => Some(Object::Builtin(println)),
        "readln" => Some(Object::Builtin(readln)),
        // "len" => Some(Object::Builtin(len)),
        // "first" => Some(Object::Builtin(first)),
        // "last" => Some(Object::Builtin(last)),
        // "rest" => Some(Object::Builtin(rest)),
        // "push" => Some(Object::Builtin(push)),
        // "int" => Some(Object::Builtin(int)),
        &_ => None,
    }
}

fn args(args: Box<[Object]>) -> Result<Object, String> {
    if args.len() != 0 {
        return Err(format!("wrong number of arguments. got={}, want=0", args.len()));
    }
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    let args = args.into_iter()
        .map(|arg| Object::String(arg.into()))
        .collect::<Vec<Object>>();

    return Ok(Object::Array(Rc::from(args)));
}

fn println(args: Box<[Object]>) -> Result<Object, String> {
    for arg in args.iter() {
        println!("{}", arg.view());
    }
    let result = args.last()
        .map(|last| last.clone())
        .unwrap_or(Object::Unit);
    Ok(result)
}


fn readln(args: Box<[Object]>) -> Result<Object, String> {
    if args.len() != 0 {
        return Err(format!("wrong number of arguments. got={}, want=0", args.len()));
    }
    let mut stdin = BufReader::new(io::stdin());
    let mut line: String = String::new();
    let Ok(_) = stdin.read_line(&mut line) else {
        return Err("Failed to read line from standard input.".to_string());
    };

    Ok(Object::String(line.trim().into()))
}

