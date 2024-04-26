use std::rc::Rc;
use crate::object::object::Object;

#[cfg(test)]
mod test {
    use std::rc::Rc;
    use crate::evaluator::evaluator::Eval;
    use crate::lexer::lexer::Lexer;
    use crate::object::environment::Environment;
    use crate::object::object::Object;
    use crate::parser::parser::Parser;

    #[test]
    fn test_eval_integer_expressions() {
        let tests = [
            ("5", 5),
            ("10", 10),
            ("(+ 5 5)", 10),
            ("-5", -5),
            ("-10", -10),
            ("(- 5)", -5),
            ("(--10)", 10),
            ("(- -10)", 10),
            ("(+ 1 2 3)", 6),
            ("(- 6 3 1)", 2),
            ("(* 2 2 3)", 12),
            ("(/ 6 3 1)", 2),
        ];

        for (input, expected) in tests {
            let evaluated = apply_eval(input).unwrap();
            test_integer_object(evaluated, expected, input);
        }
    }

    #[test]
    fn test_eval_float_expressions() {
        let tests = [
            ("5.", 5.0),
            ("10.0", 10.0),
            ("(+ 5. 5)", 10.),
            ("-5.", -5.),
            ("-10.0", -10.0),
            ("(- 5.)", -5.),
            ("(--10.)", 10.),
            ("(- -10.)", 10.),
            ("(+ 1 2. 3)", 6.),
            ("(- 6. 3 1)", 2.),
            ("(* 2 2 3.)", 12.),
            ("(/ 6 3. 1)", 2.),
            ("(/ 1 2.)", 0.5),
            ("(/ 1. 2)", 0.5),
            ("(/ 2)", 0.5),
            ("(/ 5 2)", 2.5),
        ];

        for (input, expected) in tests {
            let evaluated = apply_eval(input).unwrap();
            test_float_object(evaluated, expected, input);
        }
    }

    #[test]
    fn test_eval_boolean_expressions() {
        let tests = [
            ("true", true),
            ("false", false),
            ("(!false)", true),
            ("(!true)", false),
            ("(! (false))", true),
            ("(! (true))", false),
            ("(! 0)", true),
            ("(! 1)", false),
            ("(! 0.0)", true),
            ("(! 1.1)", false),
            ("(=)", false),
            ("(= 1)", true),
            ("(= 1 1)", true),
            ("(= 1 2)", false),
            ("(= 1 2 3)", false),
            ("(= 1 1 1)", true),
            ("(<)", false),
            ("(< 1)", true),
            ("(< 1 2)", true),
            ("(< 2 1)", false),
            ("(< 1 2 3)", true),
            ("(< 3 2 1)", false),
            ("(>)", false),
            ("(> 1)", true),
            ("(> 1 2)", false),
            ("(> 2 1)", true),
            ("(> 1 2 3)", false),
            ("(> 3 2 1)", true),
        ];

        for (input, expected) in tests {
            let evaluated = apply_eval(input).unwrap();
            test_boolean_object(evaluated, expected, input);
        }
    }

    #[test]
    fn test_eval_s_expressions() {
        let input = "(1 2 3)";

        let evaluated = apply_eval(input).unwrap();
        test_integer_object(evaluated, 3, input);
    }

    #[test]
    fn test_eval_empty_s_expressions() {
        let input = "()";

        let evaluated = apply_eval(input).unwrap();
        assert_eq!(Object::Unit, evaluated);
    }

    #[test]
    fn test_eval_string() {
        let input = "\"This is text\"";

        let evaluated = apply_eval(input).unwrap();
        assert_eq!(Object::String(Rc::from("This is text")), evaluated);
    }

    #[test]
    fn test_eval_string_concatenation() {
        let tests = [
            ("(+ \"Hello\" \" \" \"World\")", "Hello World"),
            ("(+ \"Value (\" () \")\")", "Value (())"),
            ("(+ \"Value (\" 1 \")\")", "Value (1)"),
            ("(+ \"Value (\" 3.7 \")\")", "Value (3.7)"),
            ("(+ \"Value (\" true \")\")", "Value (true)"),
            ("(+ \"Result: \" (+ 1 1) \"!\")", "Result: 2!"),
            ("(+ \"Result: \" (< 1 2) \"!\")", "Result: true!"),
        ];

        for (input, expected) in tests {
            let evaluated = apply_eval(input).unwrap();
            assert_eq!(Object::String(Rc::from(expected)), evaluated);
        }
    }

    #[test]
    fn test_array_literal() {
        let tests = [
            ("[(/ 2 2) (+ 1 1) 3]", Object::Array(Rc::from([Object::Integer(1), Object::Integer(2), Object::Integer(3)]))),
            ("[1 2 3]", vec![Object::Integer(1), Object::Integer(2), Object::Integer(3)].into()),
            ("[1 7.4 3]", vec![Object::Integer(1), Object::Float(7.4), Object::Integer(3)].into()),
            ("[1 7.4 true]", vec![Object::Integer(1), Object::Float(7.4), Object::Boolean(true)].into()),
            ("[() 4 true]", vec![Object::Unit, Object::Integer(4), Object::Boolean(true)].into()),
            ("[\"text\" 4 true]", vec![Object::String("text".into()), Object::Integer(4), Object::Boolean(true)].into()),
            ("[\"text\" \" \" \"string\"]", vec![Object::String("text".into()), Object::String(" ".into()), Object::String("string".into())].into()),
        ];


        for (input, expected) in tests {
            let evaluated = apply_eval(input).unwrap();
            assert_eq!(expected, evaluated);
        }
    }

    #[test]
    fn test_array_index_operator() {
        let tests = [
            ("(@ 1 [\"one\" \"two\" \"three\"])", Object::String(Rc::from("two"))),
            ("(set (foo [5 6 7]))(@ 1 foo)", Object::Integer(6)),
        ];

        for (input, expected) in tests {
            let evaluated = apply_eval(input).unwrap();
            assert_eq!(expected, evaluated);
        }
    }

    #[test]
    fn test_if_expression() {
        let tests = [
            ("(if (true) \"hello\")", Object::String(Rc::from("hello"))),
            ("(if (false) \"hello\")", Object::Boolean(false)),
            ("(set (a 7))(if (! (= 1 2)) a)", Object::Integer(7)),
            ("(if (true) 1)", Object::Integer(1)),
            ("(if (false) 1)", Object::Boolean(false)),
            ("(if (0) 1)", Object::Integer(0)),
            ("(if (0.0) 1)", Object::Float(0.)),
            ("(if (< 3 4) 1 2)", Object::Integer(1)),
            ("(if (< 4 3) 1 2)", Object::Integer(2)),
            ("(if (> 4 3) (+ 3 3) (* 3 3))", Object::Integer(6)),
            ("(if (> 3 4) (+ 3 3) (* 3 3))", Object::Integer(9)),
        ];

        for (input, expected) in tests {
            let evaluated = apply_eval(input).unwrap();
            assert_eq!(expected, evaluated, "Failed to evaluate: {input}");
        }
    }

    #[test]
    fn test_while_expression() {
        let tests = [
            ("(while (false) \"hello\")", Object::Boolean(false)),
            ("(while (0) 1)", Object::Integer(0)),
            ("(while (0.0) 1)", Object::Float(0.)),
            ("(set (res 0)) (set (run 5)) (while (run) ((set (res (+ res 2))) (set (run (- run 1))) )) (res)", Object::Integer(10)),
            ("(set (res 0)) (set (run 5)) (while (> run 2) ((set (res (+ res 2))) (set (run (- run 1))) )) (res)", Object::Integer(6)),
            ("(set (x 3)) (while (set (x (- x 1))))", Object::Integer(0)),
        ];

        for (input, expected) in tests {
            let evaluated = apply_eval(input).unwrap();
            assert_eq!(expected, evaluated, "Failed to evaluate: {input}");
        }
    }

    #[test]
    fn test_set_expression() {
        let tests = [
            ("(set (x 3)) (x)", Object::Integer(3)),
            ("(set (x 3) (y 7)) (x)", Object::Integer(3)),
            ("(set (x 3) (y 7)) (y)", Object::Integer(7)),
        ];

        for (input, expected) in tests {
            let evaluated = apply_eval(input).unwrap();
            assert_eq!(expected, evaluated, "Failed to evaluate: {input}");
        }
    }

    fn apply_eval(input: &str) -> Result<Object, String> {
        // let program = Parser::new(Lexer::new(input)).parse_program();
        let lexer = Lexer::from(input);
        let parser = Parser::from(lexer);
        let program = parser.parse_program().unwrap();

        program.eval(&mut Environment::new())
    }

    // fn test_integer_object(object: Object, expected: i64, input: &str) {
    fn test_integer_object(object: Object, expected: i32, input: &str) {
        let Object::Integer(actual) = object else {
            panic!("object is not Integer. got={:?}", object)
        };
        assert_eq!(expected, actual, "Input '{input}' failed to validate");
    }

    fn test_float_object(object: Object, expected: f64, input: &str) {
        let Object::Float(actual) = object else {
            panic!("object is not Integer. got={:?}", object)
        };
        assert_eq!(expected, actual, "Input '{input}' failed to validate");
    }

    fn test_boolean_object(object: Object, expected: bool, input: &str) {
        let Object::Boolean(actual) = object else {
            panic!("object is not Integer. got={:?}", object)
        };
        assert_eq!(expected, actual, "Input '{input}' failed to validate");
    }
}

impl From<Vec<Object>> for Object {
    fn from(value: Vec<Object>) -> Self {
        Object::Array(Rc::from(value))
    }
}