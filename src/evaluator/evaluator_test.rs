use std::rc::Rc;
use crate::object::object::Object;

#[cfg(test)]
mod test {
    use std::rc::Rc;
    use crate::evaluator::error::EvaluationError;
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
    fn test_spread_operator() {
        let tests = [
            ("..[7 5 2]", Object::Spread([Object::Integer(7), Object::Integer(5), Object::Integer(2)].into())),
            ("(..[7 5 2])", Object::Integer(2)),
            ("(10 ..[7 5 2])", Object::Integer(2)),
            ("(..[7 5 2] 10)", Object::Integer(10)),
            ("(..[])", Object::Unit),
            ("(set (arr [7 5 2])) ..arr", Object::Spread([Object::Integer(7), Object::Integer(5), Object::Integer(2)].into())),
            ("(set (arr [7 5 2])) (..arr)", Object::Integer(2)),
            ("(set (arr ..[7 5 2])) arr", Object::Integer(2)),
            ("(set (res (..[7 5 2]))) res", Object::Integer(2)),
            ("(+ ..[7 5 2])", Object::Integer(14)),
            ("(+ 7 ..[5 2])", Object::Integer(14)),
            ("(+ ..[7 5] 2)", Object::Integer(14)),
            ("(- ..[13 5 2])", Object::Integer(6)),
            ("(- 13 ..[5 2])", Object::Integer(6)),
            ("(- ..[13 5] 2)", Object::Integer(6)),
            ("(* ..[7 5 2])", Object::Integer(70)),
            ("(* 7 ..[5 2])", Object::Integer(70)),
            ("(* ..[7 5] 2)", Object::Integer(70)),
            ("(/ ..[16 4 2])", Object::Integer(2)),
            ("(/ 16 ..[4 2])", Object::Integer(2)),
            ("(/ ..[16 4] 2)", Object::Integer(2)),
            ("(< ..[6 14 22])", Object::Boolean(true)),
            ("(< 6 ..[14 22])", Object::Boolean(true)),
            ("(< ..[6 14] 22)", Object::Boolean(true)),
            ("(> ..[16 4 2])", Object::Boolean(true)),
            ("(> 16 ..[4 2])", Object::Boolean(true)),
            ("(> ..[16 4] 2)", Object::Boolean(true)),
            ("(= ..[3 3 3])", Object::Boolean(true)),
            ("(= 3 ..[3 3])", Object::Boolean(true)),
            ("(= ..[3 3] 3)", Object::Boolean(true)),
            ("(! ..[true])", Object::Boolean(false)),
            ("(! ..[false])", Object::Boolean(true)),
            ("(! ..[1])", Object::Boolean(false)),
            ("(! ..[\"text\"])", Object::Boolean(false)),
            ("[..[1 2] ..[3 4]]", vec![Object::Integer(1), Object::Integer(2), Object::Integer(3), Object::Integer(4)].into()),
            ("(set (a [1 2]) (b [2 3])) [..[1 2] ..[3 4]]", vec![Object::Integer(1), Object::Integer(2), Object::Integer(3), Object::Integer(4)].into()),
            ("(set (func (fn |a b| b)) (arr [2 3])) (func ..arr) ", Object::Integer(3)),
            ("(set (func (fn |a b| b))) (func ..[2 3])", Object::Integer(3)),
        ];

        for (input, expected) in tests {
            let evaluated = apply_eval(input).expect(format!("Coult not evaulate {input} Expected {expected}").as_str());
            assert_eq!(expected, evaluated, "Failed at {}", input);
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
    fn test_when_expression() {
        let tests = [
            ("(when (true) \"hello\")", Object::String(Rc::from("hello"))),
            ("(when (false) \"hello\")", Object::Boolean(false)),
            ("(set (a 7))(when (! (= 1 2)) a)", Object::Integer(7)),
            ("(when (true) 1)", Object::Integer(1)),
            ("(when (false) 1)", Object::Boolean(false)),
            ("(when (0) 1)", Object::Integer(0)),
            ("(when (0.0) 1)", Object::Float(0.)),
            ("(when (< 3 4) 1 () 2)", Object::Integer(1)),
            ("(when (< 4 3) 1 () 2)", Object::Integer(2)),
            ("(when (> 4 3) (+ 3 3) () (* 3 3))", Object::Integer(6)),
            ("(when (> 3 4) (+ 3 3) (true) (* 3 3))", Object::Integer(9)),
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

    #[test]
    fn test_include_expression() {
        let tests = [
            ("(include \"samples/seven.kvist\")", Object::Integer(7)),
            ("(include \"samples/seven.kvist\")(+ \"Seven: \" seven)", Object::String("Seven: 7".into())),
        ];

        for (input, expected) in tests {
            let evaluated = apply_eval(input).unwrap();
            assert_eq!(expected, evaluated, "Failed to evaluate: {input}");
        }
    }

    #[test]
    fn test_recursion() {
        let tests = [
            ("(set (fact (fn |num| ( (when (= num 1) 1 () (* num (fact (- num 1)))))))) (fact 3)",
                Object::Integer(6))
        ];

        for (input, expected) in tests {
            let evaluated = apply_eval(input).unwrap();
            assert_eq!(expected, evaluated, "Failed to evaluate: {input}");
        }
    }

    #[test]
    fn test_vararg_function() {
        let tests = [
            ("(set (myfunction (fn |a b ...c| (+ a b (len c))))) (myfunction 1 2 3 4)",
             Object::Integer(5)),
            ("(set (myfunction (fn |a b ...c| (+ a b (len c))))) (myfunction 1 2)",
                 Object::Integer(3))
        ];

        for (input, expected) in tests {
            let evaluated = apply_eval(input).unwrap();
            assert_eq!(expected, evaluated, "Failed to evaluate: {input}");
        }
    }

    fn apply_eval(input: &str) -> Result<Object, EvaluationError> {
        // let program = Parser::new(Lexer::new(input)).parse_program();
        let lexer = Lexer::from(input);
        let parser = Parser::from(lexer);
        let program = parser.parse_program().expect(format!("Failed to parse program: {}", input).as_str());

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