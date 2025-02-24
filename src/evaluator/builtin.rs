use crate::object::object::{Object, Viewable};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::{env, io};

pub fn builtins(name: &str) -> Option<Object> {
    match name {
        "args" => Some(Object::Builtin(args)),
        "println" => Some(Object::Builtin(println)),
        "readln" => Some(Object::Builtin(readln)),
        "len" => Some(Object::Builtin(len)),
        "first" => Some(Object::Builtin(first)),
        "last" => Some(Object::Builtin(last)),
        "rest" => Some(Object::Builtin(rest)),
        "push" => Some(Object::Builtin(push)),
        "parse_int" => Some(Object::Builtin(parse_int)),
        "os_execute" => Some(Object::Builtin(os_execute)),
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

fn len(args: Box<[Object]>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err(format!("wrong number of arguments. got={}, want=1", args.len()));
    }
    match &args[0] {
        Object::String(string) => Ok(Object::Integer(string.len() as i32)),
        Object::Array(array) => Ok(Object::Integer(array.len() as i32)),
        _ => Err(format!("argument to `len` not supported, got {}", &args[0])),
    }
}

fn first(args: Box<[Object]>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err(format!("wrong number of arguments. got={}, want=1", args.len()));
    }

    match &args[0] {
        Object::Array(array) => Ok(array.first().map(|object| object.clone()).unwrap_or(Object::Unit)),
        _ => Err(format!("argument to `first` must be Array, got {}", &args[0])),
    }
}

fn last(args: Box<[Object]>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err(format!("wrong number of arguments. got={}, want=1", args.len()));
    }

    match &args[0] {
        Object::Array(array) => Ok(array.last().map(|object| object.clone()).unwrap_or(Object::Unit)),
        _ => Err(format!("argument to `last` must be Array, got {}", &args[0])),
    }
}

fn rest(args: Box<[Object]>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err(format!("wrong number of arguments. got={}, want=1", args.len()));
    }

    match &args[0] {
        Object::Array(array) => Ok(
            array.get(1..)
                .map(|slice| Object::Array(Rc::from(slice)))
                .unwrap_or(Object::Array([].into()))
        ),
        _ => Err(format!("argument to `rest` must be Array, got {}", &args[0])),
    }
}

fn push(args: Box<[Object]>) -> Result<Object, String> {
    if args.len() != 2 {
        return Err(format!("wrong number of arguments. got={}, want=2", args.len()));
    }

    match &args[0] {
        Object::Array(array) => {
            let mut new = array.to_vec();
            new.push(args[1].clone());
            Ok(Object::Array(Rc::from(new)))
        },
        _ => Err(format!("argument to `push` must be Array, got {}", &args[0])),
    }
}

fn parse_int(args: Box<[Object]>) -> Result<Object, String> {
    if args.len() != 1 {
        return Err(format!("wrong number of arguments. got={}, want=1", args.len()));
    }

    match &args[0] {
        Object::String(string) => {
            let Ok(int) = string.parse::<i32>() else {
                return Err(format!("Number format error. Cannot convert \"{string}\" to int."));
            };
            Ok(Object::Integer(int))
        },
        object @ _ => Err(format!("Cannot convert {} to int", object))
    }
}

fn os_execute(args: Box<[Object]>) -> Result<Object, String> {
    if args.is_empty() {
        return Err("no command to execute".to_string());
    }

    fn valid_command(arg: &Object) -> Result<String, String> {
        match arg {
            object @ Object::String(_) => Ok(object.view()),
            object @ Object::Boolean(_) => Ok(object.view()),
            object @ Object::Float(_) => Ok(object.view()),
            object @ Object::Integer(_) => Ok(object.view()),
            _ => Err(format!("Invalid command '{}' not allowed.", arg.view())),
        }
    }

    let command = valid_command(&args[0])?;

    let mut command_arguemets = Vec::new();
    for arg in &args[1..] {
        command_arguemets.push(valid_command(arg)?);
    }

    let result = match Command::new(command)
        .args(command_arguemets)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .output() {
        Ok(output) => output.status.code()
            .map_or(Object::Unit, |code| Object::Integer(code)),
        Err(e) => return Err(format!("Command failed to execute '{}'", e)),
    };

    Ok(result)
}