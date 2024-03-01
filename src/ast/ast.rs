use crate::ast::expression::Expression;
use crate::token::token::Token;


pub trait AST {

    fn token_literal(&self) -> &str;

    fn string(&self) -> Box<str>;
}

pub struct Program {
    pub nodes: Box<[Node]>
}

impl From<Vec<Node>> for Program {
    fn from(nodes: Vec<Node>) -> Self {
        Program {
            nodes: nodes.into_boxed_slice()
        }
    }
}

impl AST for Program {
    fn token_literal(&self) -> &str {
        if self.nodes.is_empty() {
            return "";
        }
        return self.nodes[0].token_literal();
    }

    fn string(&self) -> Box<str> {
        let mut out = String::new();

        for node in self.nodes.iter() {
            out.push_str(&node.string());
        }

        Box::from(out)
    }
}

#[derive(Debug, PartialEq)]
pub struct Node {
    pub expression: Expression,
    // col: u32,
    // row: u32,
    pub(crate) token: Token,
}

impl AST for Node {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> Box<str> {
        self.expression.string(&self.token.literal)
    }
}
