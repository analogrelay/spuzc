use crate::{
    ast::Decl,
    parser::{prelude::*, tokens, PResult, Span, Token},
};

pub fn decl(inp: Span) -> IResult<Span, PResult<Token<Decl>> {
    func(inp)
}