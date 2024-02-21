

#[cfg(test)]
mod test {
    use crate::ast::expression::Expression;
    use crate::lexer::lexer::Lexer;
    use crate::parser::parser::Parser;

    #[test]
    fn test_let_expression() {
        let tests = [
            ("(let (x 5))", "x", 5.expect()),
            ("(let (y true))", "y", true.expect()),
            ("(let (foobar y))", "foobar", "y".expect()),
        ];


        for (input, expected_identifier, expected_value) in tests {
            // let lexer = Lexer::new(input);
            // println!("{}", input);
            let lexer = Lexer::from(input);
            let parser = Parser::from(lexer);

            // let (program, errors) = parser.parse_program();
            let program = parser.parse_program().unwrap();
            assert_eq!(1, program.nodes.len(), "Expected 1 node in program for inpu: {input}");

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

    enum Expected {
        Integer(i64),
        Boolean(bool),
        String(&'static str),
    }

    fn assert_expression(expected: &Expected, expression: &Expression) {
        match expected {
            Expected::Integer(value) => assert_eq!(*value, expression.into()),
            Expected::Boolean(value) => assert_eq!(*value, expression.into()),
            Expected::String(value) => assert_eq!(*value, String::from(expression))
        };
    }

    trait Expect {
        fn expect(self) -> Expected;
    }

    impl Expect for i64 {
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

    impl From<&Expression> for i64 {
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