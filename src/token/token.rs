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
    Float,
    String,

    // Operators
    Comma,
    DoubleColon,
    DoubleDot,
    Ellipsis,
    At,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    LesserThan,
    GreaterThan,

    Equals,

    //Delimiters
    Section,
    Pipe,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,


    // Keywords
    Function,
    Set,
    True,
    False,
    If,
    When,
    While,
}