use crate::text::Span;

#[derive(Debug, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Colon,

    Func,
    Int,

    Identifier(String),
    Integer(i128),
}

pub trait SyntaxNode {
    type Children: SyntaxNode;
    type Iter: Iterator<Item = Self::Children>;

    fn span(&self) -> Span;
    fn children(&self) -> Self::Iter;
}
