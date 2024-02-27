

#[cfg(test)]
mod parser_test {
    use crate::ast::ast::Node;
    use crate::ast::expression::Expression;
    use crate::lexer::lexer::Lexer;
    use crate::parser::parser::Parser;

    #[test]
    fn test_let_expression() {
        let tests = [
            ("(let (x 5))", "x", 5.expect()),
            ("(let (y true))", "y", true.expect()),
            ("(let (foobar y))", "foobar", Expected::Identifier("y")),
        ];


        for (input, expected_identifier, expected_value) in tests {
            // let lexer = Lexer::new(input);
            println!("{}", input);
            let lexer = Lexer::from(input);
            let parser = Parser::from(lexer);

            // let (program, errors) = parser.parse_program();
            let program = parser.parse_program().unwrap();
            assert_eq!(1, program.nodes.len(), "Expected 1 node in program for input: {input}");

            // let Expression::SExpression(nodes) = &program.nodes[0].expression else {
            //     panic!("Expected let-expression got={:?}", program.nodes[0].expression);
            // };

            let Expression::Let(name, value) = &program.nodes[0].expression else {
                panic!("Expected let-expression got={:?}", program.nodes[0].expression);
            };

            let Expression::Identifier(ref name) = name.expression else {
                panic!("Expected identifier-expression got={:?}", name.expression);
            };

            assert_eq!(expected_identifier, name.as_ref());
            assert_expression(&expected_value, &value.expression);
        }
    }

    #[test]
    fn test_parsing_prefix_expression() {
        let prefix_test = [
            ("(+ 1 2 3)", "+", [1.expect(), 2.expect(), 3.expect()]),
            ("(- 1 2 3)", "-", [1.expect(), 2.expect(), 3.expect()]),
            ("(* 1 2 3)", "*", [1.expect(), 2.expect(), 3.expect()]),
            ("(/ 1 2 3)", "/", [1.expect(), 2.expect(), 3.expect()]),
        ];

        for (input, expected_operator, expected_operands) in prefix_test {
            let lexer = Lexer::from(input);
            let parser = Parser::from(lexer);
            let program = parser.parse_program().unwrap();

            assert_eq!(1, program.nodes.len(), "Expected 1 node in program for input: {input}");

            let Expression::Prefix(operator, operands) = &program.nodes[0].expression else {
                panic!("Expected prefix-expression got={:?}", program.nodes[0].expression);
            };

            assert_eq!(expected_operator, operator.as_ref());
            assert_nodes(expected_operands.as_ref(), operands);
        }
    }

    #[test]
    fn test_float_expression() {
        let tests = [
            // ("(let (x 5.))", 5.),
            // ("(6.6)", 6.6),
            ("6.", 6.),
            ("7.7", 7.7),
        ];

        for (input, expected) in tests {
            let lexer = Lexer::from(input);
            let parser = Parser::from(lexer);
            let program = parser.parse_program().unwrap();

            assert_eq!(1, program.nodes.len(), "Expected 1 node in program for input: {input}");

            let Expression::Float(value) = &program.nodes[0].expression else {
                panic!("Expected prefix-expression got={:?}", program.nodes[0].expression);
            };

            assert_eq!(expected, *value);
        }
    }

    #[test]
    fn test_s_expression() {
        let input = "(1 2)";
        let lexer = Lexer::from(input);
        let parser = Parser::from(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(1, program.nodes.len(), "Expected 1 node in program for input: {input}");

        let Expression::SExpression(nodes) = &program.nodes[0].expression else {
            panic!("Expected prefix-expression got={:?}", program.nodes[0].expression);
        };
        assert_eq!(2, nodes.len());

        let Expression::Integer(element) = nodes[0].expression else {
            panic!("Expected integer-expression got={:?}", program.nodes[0].expression);
        };
        assert_eq!(1, element);

        let Expression::Integer(element) = nodes[1].expression else {
            panic!("Expected integer-expression got={:?}", program.nodes[0].expression);
        };
        assert_eq!(2, element);
    }

    #[test]
    fn test_empty_s_expression() {
        let input = "()";
        let lexer = Lexer::from(input);
        let parser = Parser::from(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(1, program.nodes.len(), "Expected 1 node in program for input: {input}");

        let Expression::SExpression(value) = &program.nodes[0].expression else {
            panic!("Expected SExpression expression got={:?}", program.nodes[0].expression);
        };
        assert!(value.is_empty())
    }


    enum Expected {
        // Integer(i64),
        Integer(i32),
        Boolean(bool),
        String(&'static str),
        Identifier(&'static str),
    }

    fn assert_expression(expected: &Expected, expression: &Expression) {
        match expected {
            Expected::Integer(expected) => assert_eq!(*expected, expression.into()),
            Expected::Boolean(expected) => assert_eq!(*expected, expression.into()),
            Expected::Identifier(expected) => assert_identifier(*expected, expression),
            Expected::String(expected) => assert_eq!(*expected, String::from(expression))
        };
    }

    fn assert_identifier(expected: &str, expression: &Expression) {
        let Expression::Identifier(identifier) = expression else {
            panic!("expression is not Identifier. got={:?}", expression)
        };
        assert_eq!(expected, identifier.as_ref());
    }

    fn assert_nodes(expected: &[Expected], nodes: &[Node]) {
        assert_eq!(expected.len(), nodes.len(), "nodes does not match expected length");
        for (index, expected) in expected.iter().enumerate() {
            assert_expression(expected, &nodes[index].expression);
        }
    }

    trait Expect {
        fn expect(self) -> Expected;
    }

    // impl Expect for i64 {
    impl Expect for i32 {
        fn expect(self) -> Expected {
            Expected::Integer(self)
        }
    }

    impl Expect for bool {
        fn expect(self) -> Expected {
            Expected::Boolean(self)
        }
    }

    impl Expect for &'static str {
        fn expect(self) -> Expected {
            Expected::String(self)
        }
    }

    // impl From<i64> for Expected {
    //     fn from(value: i64) -> Self {
    //         Expected::Integer(value)
    //     }
    // }
    //
    // impl From<bool> for Expected {
    //     fn from(value: bool) -> Self {
    //         Expected::Boolean(value)
    //     }
    // }
    //
    // impl From<&'static str> for Expected {
    //     fn from(value: &'static str) -> Self {
    //         Expected::String(value)
    //     }
    // }

    // impl From<&Expression> for i64 {
    impl From<&Expression> for i32 {
        fn from(expression: &Expression) -> Self {
            let Expression::Integer(value) = expression else {
                panic!("Expected Integer got={:?}", expression)
            };
            return *value;
        }
    }

    impl From<&Expression> for bool {
        fn from(expression: &Expression) -> Self {
            let Expression::Boolean(value) = expression else {
                panic!("Expected Boolean got={:?}", expression)
            };
            return *value;
        }
    }

    impl From<&Expression> for String {
        fn from(expression: &Expression) -> Self {
            let Expression::String(value) = expression else {
                panic!("Expected String got={:?}", expression)
            };
            return value.to_string();
        }
    }
}