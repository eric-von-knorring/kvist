use std::rc::Rc;
use std::str::CharIndices;

use crate::token::token::{Token, TokenType};

#[derive(Debug)]
pub struct Lexer<'a> {
    col: u32,
    row: u32,
    input: CharIndices<'a>,
    position: usize,
    // read_position: u32,
    current: char,
    peek: Option<(usize, char)>,
    eof: bool,
}

impl<'a> From<&'a str> for Lexer<'a> {
    fn from(input: &'a str) -> Self {
        let mut lexer = Lexer {
            col: 1,
            row: 1,
            input: input.char_indices(),
            position: 0,
            // read_position: 0,
            current: '\0',
            peek: Some((0, '\0')),
            eof: false,
        };

        for i in 0..2 {
            (_, lexer.current) = lexer.peek.unwrap_or((i, '\0'));
            lexer.peek = lexer.input.next();
        }

        return lexer;
    }
}

impl Lexer<'_> {
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let token = match self.current {
            // '(' => Token { token_type: TokenType::LParen, row: self.row, col: self.col, literal: self.current.into()},
            ':' => self.colon_token(),
            ',' => self.create_token(TokenType::Comma, self.current.literal()),
            '§' => self.create_token(TokenType::Section, self.current.literal()),
            '!' => self.create_token(TokenType::Bang, self.current.literal()),
            '|' => self.create_token(TokenType::Pipe, self.current.literal()),
            '(' => self.create_token(TokenType::LParen, self.current.literal()),
            ')' => self.create_token(TokenType::RParen, self.current.literal()),
            '@' => self.create_token(TokenType::At, self.current.literal()),
            '+' => self.create_token(TokenType::Plus, self.current.literal()),
            '-' => return self.read_minus(),
            '*' => self.create_token(TokenType::Asterisk, self.current.literal()),
            '/' => self.create_token(TokenType::Slash, self.current.literal()),
            '=' => self.create_token(TokenType::Equals, self.current.literal()),
            '{' => self.create_token(TokenType::LBrace, self.current.literal()),
            '}' => self.create_token(TokenType::RBrace, self.current.literal()),
            '[' => self.create_token(TokenType::LBracket, self.current.literal()),
            ']' => self.create_token(TokenType::RBracket, self.current.literal()),
            '<' => self.create_token(TokenType::LesserThan, self.current.literal()),
            '>' => self.create_token(TokenType::GreaterThan, self.current.literal()),
            '"' => self.read_string(),
            '\0' => {
                self.eof = true;
                self.create_token(TokenType::EOF, Rc::from(""))
            }
            _ => {
                if Lexer::is_letter(self.current) {
                    return self.read_identifier()
                } else if self.current.is_digit(10)  {
                    // return self.read_number()
                    return self.read_number(String::new())
                } else {
                    self.create_token(TokenType::Illegal, self.current.literal())
                }
            }
        };
        self.read_char();
        return token;
    }

    // fn create_token<'a>(&self, token_type: TokenType, literal: &'a str) -> Token<'a> {
    fn create_token(&self, token_type: TokenType, literal: Rc<str>) -> Token {
        Token { token_type, row: self.row, col: self.col, literal }
    }

    fn read_identifier(&mut self) -> Token {
        // let position = self.position;
        let col = self.col;
        let row = self.row;
        let mut literal = String::new();
        while Lexer::is_letter(self.current) {
            literal.push(self.current);
            self.read_char();
        }
        // let literal = &self.input[position..self.position];
        // return self.identifier_token(Rc::from(literal));
        let literal = Rc::from(literal);
        return Token { token_type: self.identifier_token_type(&literal), col, row, literal}
    }

    fn identifier_token_type(&self, literal: &str) -> TokenType {
        match literal {
            "let" => TokenType::Let,
            "fn" => TokenType::Function,
            "true" => TokenType::True,
            "false" => TokenType::False,
            _ => TokenType::Ident,
        }
    }

    fn read_string(&mut self) -> Token {
        let row = self.row;
        let col = self.col;

        let mut string = String::new();
        // self.read_char();
        loop {
            self.read_char();
            if self.current == '"' || self.current == '\0' {
                break;
            }
            string.push(self.current);
        }
        // return self.input[position..self.position].to_string();
        return Token {
            token_type: TokenType::String,
            row,
            col,
            literal: Rc::from(string),
        };
    }

    fn read_minus(&mut self) -> Token {
        let current = self.current;
        self.read_char();
        if self.current.is_digit(10) {
            return self.read_number(String::from(current));
        }
        self.create_token(TokenType::Minus, current.literal())
    }

    fn read_number(&mut self, literal: String) -> Token {
        let mut token_type = TokenType::Int;
        let mut literal = self.read_digits(literal);
        if self.current == '.' {
            token_type = TokenType::Float;
            literal.push(self.current);
            self.read_char();
            literal = self.read_digits(literal);
        }
        return self.create_token(token_type, Rc::from(literal));
    }

    fn read_digits(&mut self, mut literal: String) -> String {
        while self.current.is_digit(10) {
            literal.push(self.current);
            self.read_char();
        }
        return literal;
    }

    fn is_letter(character: char) -> bool {
        character.is_alphabetic() || character == '_'
    }

    fn colon_token(&mut self) -> Token {
        if let Some((_, ':')) = self.peek {
            let col = self.col;
            let row = self.row;
            let mut literal = String::new();
            literal.push(self.current);
            self.read_char();
            literal.push(self.current);
            // return self.create_token(TokenType::DoubleColon, Rc::from(literal));
            return Token {token_type: TokenType::DoubleColon, col, row, literal: Rc::from(literal)};
        }
        return self.create_token(TokenType::Illegal, self.current.literal());
    }

    fn read_char(&mut self) {
        // let Some((index, current)) = self.input.next() else {
        let Some((index, current)) = self.peek else {
            self.current = '\0';

            if !self.eof { self.position += 1 };

            self.eof = true;
            return;
        };
        self.peek = self.input.next();

        self.current = current;
        self.position = index;
        // println!("{}", current);
        self.col += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.current.is_whitespace() {
            if self.current == '\n' {
                self.row += 1;
                self.col = 0;
            }
            self.read_char();
        }
    }
}

trait ToLiteral {

    fn literal(self) -> Rc<str>;
}

impl ToLiteral for char {
    fn literal(self) -> Rc<str> {
        Rc::from(self.to_string().as_str())
    }
}
