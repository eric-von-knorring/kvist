use std::rc::Rc;



#[derive(PartialEq, Eq, Debug)]
pub struct Token {
    pub(crate) token_type: TokenType,
    pub(crate) col: u32,
    pub(crate) row: u32,
    pub(crate) literal: Rc<str>,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum TokenType {
    Illegal,
    EOF,

    // Identifiers + literals
    Ident,
    Int,
    String,

    // Operators
    Comma,
    DoubleColon,
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    LesserThan,
    GreaterThan,

    Equals,

    //Delimiters
    Builtin,
    Pipe,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,


    // Keywords
    Null,
    Function,
    Let,
    True,
    False,
    // If,
    // Else,
    // Return,
}