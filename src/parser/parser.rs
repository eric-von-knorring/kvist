use std::mem;
use crate::ast::ast::{Node, Program};
use crate::ast::expression::Expression;

use crate::lexer::lexer::Lexer;
use crate::token::token::{Token, TokenType};
use crate::token::token::TokenType::RParen;

#[derive(Debug)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    // TODO Replace with error type
    errors: Vec<String>,
    current_token: Token,
    peek_token: Token,
}

impl Parser<'_> {
    // pub fn new(mut lexer: Lexer) {
    //     let current_token = lexer.next_token();
    //     let peek_token = lexer.next_token();
    //
    //     Parser {
    //         lexer,
    //         errors: Vec::new(),
    //         current_token,
    //         peek_token,
    //     };
    // }

    // pub fn parse_program(mut self) -> (Program, Vec<String>) {
    pub fn parse_program(mut self) -> Result<Program, Vec<String>> {
        // let mut program = Program::new();
        let mut nodes = Vec::new();
        while !self.current_token_is(TokenType::EOF) {
            println!("{:?}", self.current_token);
            if let Some(node) = self.parse_expression() {
                // program.statements.push(statement);
                nodes.push(node);
            }
            // self.next_token();
        }
        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        // return Ok(program);
        return Ok(nodes.into());
    }

    fn current_token_is(&self, token_type: TokenType) -> bool {
        self.current_token.token_type == token_type
    }

    fn next_token(&mut self) -> Token {
        // let previous = self.current_token;
        // self.current_token = mem::replace(&mut self.peek_token, self.lexer.next_token());
        let next = mem::replace(&mut self.peek_token, self.lexer.next_token());
        // dbg!(mem::replace(&mut self.current_token, next))
        mem::replace(&mut self.current_token, next)
    }

    fn peek_token_is(&self, token_type: TokenType) -> bool {
        self.peek_token.token_type == token_type
    }

    fn peek_error(&mut self, expected: TokenType) {
        self.errors
            .push(format!("On row: {}, col: {}. Expected next token to be {:?}, got {:?} instead.",
                          self.peek_token.row, self.peek_token.col, expected, self.peek_token))
    }

    // fn expect_peek(&mut self, token_type: TokenType) -> bool {
    fn expect_peek(&mut self, token_type: TokenType) -> Option<Token> {
        if self.peek_token_is(token_type) {
            Some(self.next_token())
        } else {
            self.peek_error(token_type);
            None
        }
    }

    fn parse_expression(&mut self) -> Option<Node> {
        if self.current_token_is(TokenType::LParen) {
            // return None;
            self.next_token();
        }
        let mut result = self.prefix_parse()?;
        Some(result)
    }

    fn prefix_parse(&mut self) -> Option<Node> {
        match self.current_token.token_type {
            TokenType::Let => self.parse_let(),
            TokenType::Int => self.parse_integer_literal(),
            _ => {
                self.next_token();
                None
            },
        }
    }

    fn parse_let(&mut self) -> Option<Node> {
        let current = self.expect_peek(TokenType::LParen)?;

        self.expect_peek(TokenType::Ident)?;

        let identifier_token = self.next_token();
        let identifier = Node {
            expression: Expression::Identifier(identifier_token.literal.clone()),
            token: identifier_token,
        };

        println!("{:?}", self.current_token);
        // FIXME, maybe an error should be returned here.
        let value = self.parse_expression()?;

        // FIXME, should probably also be an error
        if self.peek_token_is(RParen) {
            self.next_token();
        }

        Some(Node {
            expression: Expression::Let(identifier.into(), value.into()),
            token: current
        })
    }


    fn parse_integer_literal(&mut self) -> Option<Node> {
        // let current = self.current_token.clone();
        let current = self.next_token();

        let value = if let Ok(value) = current.literal.parse::<i64>() {
            value
        } else {
            // FIXME, return an error
            self.errors
                .push(format!("could not parse {} as integer", current.literal));
            return None;
        };

        Some(Node {
            expression: Expression::Integer(value),
            token: current,
        })
    }
}

impl<'a> From<Lexer<'a>> for Parser<'a> {
    fn from(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();

        Parser {
            lexer,
            errors: Vec::new(),
            current_token,
            peek_token,
        }
    }
}