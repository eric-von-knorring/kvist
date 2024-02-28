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