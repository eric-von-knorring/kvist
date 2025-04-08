use std::mem;

use crate::ast::ast::{Node, Program};
use crate::ast::expression::Expression;
use crate::lexer::lexer::Lexer;
use crate::parser::error::ParseError;
use crate::token::token::{Token, TokenType};

#[derive(Debug)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    // TODO Replace with error type
    // errors: Vec<String>,
    errors: Vec<ParseError>,
    current_token: Token,
    peek_token: Token,
}

impl Parser<'_> {
    pub fn parse_program(mut self) -> Result<Program, Vec<ParseError>> {
        // let mut program = Program::new();
        let mut nodes = Vec::new();
        while !self.current_token_is(TokenType::EOF) {
            // println!("{:?}", self.current_token);
            // if let Ok(node) = self.parse_expression() {
            //     // program.statements.push(statement);
            //     // println!("{:?}", node);
            //     nodes.push(node);
            // } else { self.errors.push("Failed to parse expression.".to_string()) }
            // self.next_token();
            match self.parse_expression() {
                Ok(node) => nodes.push(node),
                Err(error) => self.errors.push(error)
            }
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
        let next = mem::replace(&mut self.peek_token, self.lexer.next_token());
        mem::replace(&mut self.current_token, next)
    }

    fn peek_token_is(&self, token_type: TokenType) -> bool {
        self.peek_token.token_type == token_type
    }

    fn peek_token_is_literal(&self) -> bool {
        match self.peek_token.token_type {
            TokenType::Ident
            | TokenType::Int
            | TokenType::Float
            | TokenType::True
            | TokenType::False
            | TokenType::LParen => true,
            _ => false,
        }
    }

    // fn peek_error(&mut self, expected: TokenType) {
    //     self.errors
    //         .push(format!("On row: {}, col: {}. Expected next token to be {:?}, got {:?} instead.",
    //                       self.peek_token.row, self.peek_token.col, expected, self.peek_token))
    // }

    // fn expect_peek(&mut self, token_type: TokenType) -> bool {
    fn expect_peek(&mut self, token_type: TokenType) -> Result<Token, ParseError> {
        if self.peek_token_is(token_type) {
            self.next_token().into()
        } else {
            // self.peek_error(token_type);
            // prevent infinite loops on error
            self.next_token();
            ParseError{
                col: self.peek_token.col, row: self.peek_token.row,
                message: format!("Expected next token to be {:?} but got {:?}", token_type, self.peek_token.token_type)
            }.into()
        }
    }

    fn parse_expression(&mut self) -> Result<Node, ParseError> {
        // if self.current_token_is(TokenType::LParen) && self.peek_token_is(TokenType::LParen) {
        if self.current_token_is(TokenType::EOF) {
            return ParseError{
                col: self.current_token.col, row: self.current_token.row,
                message: "Unexpected end of file".to_string()
            }.into();
        }
        if self.current_token_is(TokenType::LParen) && self.peek_token_is_literal() {
            return self.parse_expression_literal();
        }

        if self.current_token_is(TokenType::LParen) && self.peek_token_is(TokenType::RParen) {
            let current = self.next_token();
            self.next_token();
            return Node { expression: Expression::ExpressionLiteral(Box::default()), token: current }.into();
        }

        let mut in_parenthesis = false;
        if self.current_token_is(TokenType::LParen) {
            in_parenthesis = true;
            self.next_token();
        }

        let result = self.prefix_parse()?;

        match (in_parenthesis, self.current_token_is(TokenType::RParen)) {
            (true, false) => return ParseError{
                col: self.current_token.col, row: self.current_token.row,
                message: "Expected closing parenthesis".to_string(),
            }.into(),
            (true, true) => { self.next_token(); }
            (false, _) => {}
        };
        result.into()
    }

    fn prefix_parse(&mut self) -> Result<Node, ParseError> {
        match self.current_token.token_type {
            TokenType::Set => self.parse_set(),
            TokenType::If => self.parse_if(),
            TokenType::When => self.parse_when(),
            TokenType::While => self.parse_while(),
            TokenType::Include => self.parse_include(),
            TokenType::Function => self.parse_function(),
            TokenType::Section => self.parse_scoped_section(),
            TokenType::Int => self.parse_integer_literal(),
            TokenType::Float => self.parse_float_literal(),
            TokenType::String => self.parse_string_literal().into(),
            TokenType::LBracket => self.parse_array_literal(),
            TokenType::Ident => self.parse_identifier().into(),
            TokenType::At => self.parse_index_operator(),
            TokenType::DoubleDot => self.parse_spread_operator(),
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Asterisk
            | TokenType::Slash
            | TokenType::GreaterThan
            | TokenType::LesserThan
            | TokenType::Bang
            | TokenType::Equals => self.parse_prefix_operator(),
            TokenType::True | TokenType::False => self.parse_boolean().into(),
            _ => {
                let current = self.next_token();
                ParseError {
                    col: current.col, row: current.row,
                    message: format!("Could not parse prefix token type '{:?}' with literal '{}'", current.token_type, current.literal)
                }.into()
            }
        }
    }

    fn parse_expression_literal(&mut self) -> Result<Node, ParseError> {
        let token = self.next_token();
        let mut expressions = Vec::new();

        while !self.current_token_is(TokenType::RParen) {
            expressions.push(self.parse_expression()?);
        }
        self.next_token();

        Node {
            expression: Expression::ExpressionLiteral(Box::from(expressions)),
            token,
        }.into()
    }

    fn parse_set(&mut self) -> Result<Node, ParseError> {
        let current = self.expect_peek(TokenType::LParen)?;

        let mut list = Vec::new();

        while self.current_token_is(TokenType::LParen) {
            self.expect_peek(TokenType::Ident)?;

            let identifier = self.parse_identifier();

            let value = self.parse_expression()?;

            list.push((identifier, value));
            // FIXME, should probably be an error
            // self.expect_peek(TokenType::RParen)?;
            self.next_token();
        }

        Node {
            expression: Expression::Set(list.into()),
            token: current,
        }.into()
    }


    fn parse_if(&mut self) -> Result<Node, ParseError> {
        let current = self.next_token();

        let condition = self.parse_expression()?;
        let consequence = self.parse_expression()?;
        let mut alternative = None;

        if !self.current_token_is(TokenType::RParen) {
            alternative = Box::from(self.parse_expression()?).into()
        }

        Node {
            expression: Expression::If(condition.into(), consequence.into(), alternative),
            token: current,
        }.into()
    }

    fn parse_when(&mut self) -> Result<Node, ParseError> {
        let current = self.next_token();

        let mut branches = Vec::new();

        while !self.current_token_is(TokenType::RParen) {
            let condition = self.parse_expression()?;
            if self.current_token_is(TokenType::RParen) {
                return ParseError{
                    col: self.current_token.col, row: self.current_token.row,
                    message: "Expected consequence for condition in when-expression".to_string()
                }.into();
            }
            let consequence = self.parse_expression()?;
            branches.push((condition.into(), consequence.into()))
        }

        Node {
            expression: Expression::When(branches.into()),
            token: current,
        }.into()
    }

    fn parse_while(&mut self) -> Result<Node, ParseError> {
        let current = self.next_token();
        let condition = self.parse_expression()?;

        let mut loop_expression = None;
        if !self.current_token_is(TokenType::RParen) {
            loop_expression = Box::from(self.parse_expression()?).into()
        }

        Node {
            expression: Expression::While(condition.into(), loop_expression),
            token: current,
        }.into()
    }

    fn parse_include(&mut self) -> Result<Node, ParseError> {
        let current = self.next_token();
        let target = self.parse_string_literal();

        Node {
            expression: Expression::Include(target.into()),
            token: current,
        }.into()
    }

    fn parse_function(&mut self) -> Result<Node, ParseError> {
        let current = self.expect_peek(TokenType::Pipe)?;
        self.next_token();

        let mut parameters = Vec::new();
        let mut vararg = None;

        while !self.current_token_is(TokenType::Pipe) {
            match (&self.current_token.token_type, &self.peek_token.token_type) {
                (TokenType::Ellipsis, TokenType::Ident) => {
                    self.next_token();
                    let identifier = self.parse_identifier();
                    if !self.current_token_is(TokenType::Pipe) {
                        return ParseError {
                            col: self.current_token.col, row: self.current_token.row,
                            message: "Expected vararg identifier to be last in parameter list.".to_string()
                        }.into();
                    }
                    vararg = Some(identifier);
                    break;
                }
                (TokenType::Ident, _) => {
                    let param = self.parse_identifier();
                    parameters.push(param)
                }
                _ => {
                    return ParseError {
                        col: self.current_token.col, row: self.current_token.row,
                        message: "Expected function parameters names.".to_string()
                    }.into();
                }
            };
        }

        self.next_token();

        let body = self.parse_expression()?;

        Node {
            expression: Expression::Function(parameters.into(), vararg.into(), body.into()),
            token: current,
        }.into()
    }

    fn parse_scoped_section(&mut self) -> Result<Node, ParseError> {
        let current = self.next_token();

        let section = self.parse_expression()?;

        Node {
            expression: Expression::Section(section.into()),
            token: current
        }.into()
    }

    fn parse_identifier(&mut self) -> Node {
        let token = self.next_token();
        Node {
            expression: Expression::Identifier(token.literal.clone()),
            token,
        }
    }


    fn parse_integer_literal(&mut self) -> Result<Node, ParseError> {
        let current = self.next_token();

        let value = if let Ok(value) = current.literal.parse::<i32>() {
            value
        } else {
            return ParseError {
                col: current.col, row: current.row,
                message: format!("Could not parse {} as integer", current.literal)
            }.into();
        };

        Node {
            expression: Expression::Integer(value),
            token: current,
        }.into()
    }

    fn parse_float_literal(&mut self) -> Result<Node, ParseError> {
        let current = self.next_token();

        let value = if let Ok(value) = current.literal.parse::<f64>() {
            value
        } else {
            return ParseError {
                col: current.col, row: current.row,
                message: format!("Could not parse {} as float", current.literal)
            }.into();
        };

        Node {
            expression: Expression::Float(value),
            token: current,
        }.into()
    }

    fn parse_string_literal(&mut self) -> Node {
        let current = self.next_token();
        Node {
            expression: Expression::String(current.literal.clone()),
            token: current,
        }
    }

    fn parse_array_literal(&mut self) -> Result<Node, ParseError> {
        let token = self.next_token();
        let mut expressions = Vec::new();

        while !self.current_token_is(TokenType::RBracket) {
            expressions.push(self.parse_expression()?);
        }
        self.next_token();
        Node {
            expression: Expression::Array(expressions.into()),
            token,
        }.into()
    }

    fn parse_boolean(&mut self) -> Node {
        let current = self.next_token();
        Node {
            expression: Expression::Boolean(current.token_type == TokenType::True),
            token: current,
        }
    }

    fn parse_index_operator(&mut self) -> Result<Node, ParseError> {
        let current = self.next_token();
        let index = self.parse_expression()?;
        let operand = self.parse_expression()?;

        Node {
            expression: Expression::Index(index.into(), operand.into()),
            token: current,
        }.into()
    }

    fn parse_spread_operator(&mut self) -> Result<Node, ParseError> {
        let current = self.next_token();
        let target = self.parse_expression()?;

        Node {
            expression: Expression::Spread(target.into()),
            token: current,
        }.into()
    }

    fn parse_prefix_operator(&mut self) -> Result<Node, ParseError> {
        let current = self.next_token();

        let mut operands = Vec::new();

        while !self.current_token_is(TokenType::RParen) {
            operands.push(self.parse_expression()?);
        }

        Node {
            expression: Expression::Prefix(current.literal.clone(), operands.into()),
            token: current,
        }.into()
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

impl From<Token> for Result<Token, ParseError> {
    fn from(value: Token) -> Self {
        Ok(value)
    }
}

impl From<ParseError> for Result<Token, ParseError> {
    fn from(value: ParseError) -> Self {
        Err(value)
    }
}
impl From<Node> for Result<Node, ParseError> {
    fn from(value: Node) -> Self {
        Ok(value)
    }
}

impl From<ParseError> for Result<Node, ParseError> {
    fn from(value: ParseError) -> Self {
        Err(value)
    }
}
