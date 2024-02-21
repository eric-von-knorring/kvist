use std::rc::Rc;
use std::str::CharIndices;

use crate::token::token::{Token, TokenType};

#[derive(Debug)]
pub struct Lexer<'a> {
    col: u32,
    row: u32,
    input: CharIndices<'a>,
    position: usize,
    read_position: u32,
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
            read_position: 0,
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
    // pub fn new(input: &str) -> Lexer {
    //     let mut lexer = Lexer {
    //         col: 1,
    //         row: 1,
    //         input: input.char_indices(),
    //         position: 0,
    //         read_position: 0,
    //         current: '\0',
    //         peek: Some((0, '\0')),
    //         eof: false,
    //     };
    //
    //     for i in 0..2 {
    //         (_, lexer.current) = lexer.peek.unwrap_or((i, '\0'));
    //         lexer.peek = lexer.input.next();
    //     }
    //
    //     return lexer;
    // }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let token = match self.current {
            // '(' => Token { token_type: TokenType::LParen, row: self.row, col: self.col, literal: self.current.into()},
            ':' => self.colon_token(),
            ',' => self.create_token(TokenType::Comma, self.current.literal()),
            '§' => self.create_token(TokenType::Builtin, self.current.literal()),
            '|' => self.create_token(TokenType::Pipe, self.current.literal()),
            '(' => self.create_token(TokenType::LParen, self.current.literal()),
            ')' => self.create_token(TokenType::RParen, self.current.literal()),
            '+' => self.create_token(TokenType::Plus, self.current.literal()),
            '-' => self.create_token(TokenType::Minus, self.current.literal()),
            '=' => self.create_token(TokenType::Assign, self.current.literal()),
            '{' => self.create_token(TokenType::LBrace, self.current.literal()),
            '}' => self.create_token(TokenType::RBrace, self.current.literal()),
            '\0' => {
                self.eof = true;
                self.create_token(TokenType::EOF, Rc::from(""))
            }
            _ => {
                if Lexer::is_letter(self.current) {
                    return self.read_identifier()
                } else if self.current.is_digit(10)  {
                    return self.read_number()
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
            _ => TokenType::Ident,
        }
    }

    fn read_number(&mut self) -> Token {
        let mut literal = String::new();
        while self.current.is_digit(10) {
            literal.push(self.current);
            self.read_char();
        }
        return self.create_token(TokenType::Int, Rc::from(literal));
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
    // fn literal<'a>(self) -> &'a str;
    // fn literal<'a>(self) -> Box<str>;
    fn literal(self) -> Rc<str>;
}

impl ToLiteral for char {
    // fn literal<'a>(self) -> Box<str> {
    fn literal(self) -> Rc<str> {
        // let mut tmp = [0u8; 4];
        // Box::new(*self.encode_utf8(&mut tmp))
        Rc::from(self.to_string().as_str())
    }
}
