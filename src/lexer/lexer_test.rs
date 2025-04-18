

#[cfg(test)]
mod test {
    use crate::lexer::lexer::Lexer;
    use crate::token::token::TokenType;

    #[test]
    fn test_next_token_simple() {
        // let input = "=+(){},;";
        let input = "§(+)={}::";

        let expected = [
            (TokenType::Section, "§"),
            (TokenType::LParen, "("),
            (TokenType::Plus, "+"),
            (TokenType::RParen, ")"),
            // (TokenType::Assign, "="),
            (TokenType::Equals, "="),
            (TokenType::LBrace, "{"),
            (TokenType::RBrace, "}"),
            (TokenType::DoubleColon, "::"),
            // (TokenType::Comma, ","),
            // (TokenType::Semicolon, ";"),
            (TokenType::EOF, ""),
        ];

        // let mut lexer = Lexer::new(input);
        let mut lexer = Lexer::from(input);
        for (token, literal) in expected {
            let result = lexer.next_token();
            println!("{:?}", result);
            assert_eq!(token, result.token_type);
            assert_eq!(literal, &*result.literal);
        }
    }

    #[test]
    fn test_next_token() {

        let input = "\
        (set (a 5) (b 10))\
        (fn |a, b| (+ a b))\
        (- z 3)\
        (true false)\
        (* 1 2 3) \
        (/ 3 2 1) \
        (- -1) \
        (+ 5. 10.0 -10. -5.0) \
        \"This is a text\"\
        [ ! < > = ] \
        (@ 1 [1 2 3])\
        (if (true) (+ 1 1)) \
        (while (false) (\"hello\"))\
        (when (false) (\"hello\") (true) (\"world\"))\
        (.. ...)\
        (include \"file.kvist\")
        ";

        let expected = [
            (TokenType::LParen, "("),
            (TokenType::Set, "set"),
            (TokenType::LParen, "("),
            (TokenType::Ident, "a"),
            (TokenType::Int, "5"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::Ident, "b"),
            (TokenType::Int, "10"),
            (TokenType::RParen, ")"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::Function, "fn"),
            (TokenType::Pipe, "|"),
            (TokenType::Ident, "a"),
            (TokenType::Comma, ","),
            (TokenType::Ident, "b"),
            (TokenType::Pipe, "|"),
            (TokenType::LParen, "("),
            (TokenType::Plus, "+"),
            (TokenType::Ident, "a"),
            (TokenType::Ident, "b"),
            (TokenType::RParen, ")"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::Minus, "-"),
            (TokenType::Ident, "z"),
            (TokenType::Int, "3"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::True, "true"),
            (TokenType::False, "false"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::Asterisk, "*"),
            (TokenType::Int, "1"),
            (TokenType::Int, "2"),
            (TokenType::Int, "3"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::Slash, "/"),
            (TokenType::Int, "3"),
            (TokenType::Int, "2"),
            (TokenType::Int, "1"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::Minus, "-"),
            (TokenType::Int, "-1"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::Plus, "+"),
            (TokenType::Float, "5."),
            (TokenType::Float, "10.0"),
            (TokenType::Float, "-10."),
            (TokenType::Float, "-5.0"),
            (TokenType::RParen, ")"),
            (TokenType::String, "This is a text"),
            (TokenType::LBracket, "["),
            (TokenType::Bang, "!"),
            (TokenType::LesserThan, "<"),
            (TokenType::GreaterThan, ">"),
            (TokenType::Equals, "="),
            (TokenType::RBracket, "]"),
            (TokenType::LParen, "("),
            (TokenType::At, "@"),
            (TokenType::Int, "1"),
            (TokenType::LBracket, "["),
            (TokenType::Int, "1"),
            (TokenType::Int, "2"),
            (TokenType::Int, "3"),
            (TokenType::RBracket, "]"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::If, "if"),
            (TokenType::LParen, "("),
            (TokenType::True, "true"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::Plus, "+"),
            (TokenType::Int, "1"),
            (TokenType::Int, "1"),
            (TokenType::RParen, ")"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::While, "while"),
            (TokenType::LParen, "("),
            (TokenType::False, "false"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::String, "hello"),
            (TokenType::RParen, ")"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::When, "when"),
            (TokenType::LParen, "("),
            (TokenType::False, "false"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::String, "hello"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::True, "true"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::String, "world"),
            (TokenType::RParen, ")"),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::DoubleDot, ".."),
            (TokenType::Ellipsis, "..."),
            (TokenType::RParen, ")"),
            (TokenType::LParen, "("),
            (TokenType::Include, "include"),
            (TokenType::String, "file.kvist"),
            (TokenType::RParen, ")"),
            (TokenType::EOF, ""),
        ];

        // let mut lexer = Lexer::new(input);
        let mut lexer = Lexer::from(input);
        for (token, literal) in expected {
            let result = lexer.next_token();
            // println!("{:?}", result);
            assert_eq!(token, result.token_type);
            assert_eq!(literal, &*result.literal);
        }
    }
    #[test]
    fn test_col_and_row_count() {
        let input =
"(x)
a b c
:: d ::
:: :: e
.. ...";

        let expected = [
            (1, 1),
            (2, 1),
            (3, 1),
            (1, 2),
            (3, 2),
            (5, 2),
            (1, 3),
            (4, 3),
            (6, 3),
            (1, 4),
            (4, 4),
            (7, 4),
            (1, 5),
            (4, 5),
        ];

        // let mut lexer = Lexer::new(input);
        let mut lexer = Lexer::from(input);
        for expect in expected {
            let result = lexer.next_token();
            assert_eq!(expect, (result.col, result.row))
        }
    }
    #[test]
    fn test_comments_and_row_count() {
        let input =
            "#!/usr/bin/env kvist
(x)
# This is text
a b c
";

        let expected = [
            (1, 2),
            (2, 2),
            (3, 2),
            (1, 4),
            (3, 4),
            (5, 4),
        ];

        // let mut lexer = Lexer::new(input);
        let mut lexer = Lexer::from(input);
        for expect in expected {
            let result = lexer.next_token();
            assert_eq!(expect, (result.col, result.row))
        }
    }
}