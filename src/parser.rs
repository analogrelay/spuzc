use crate::{ast::Expr, tokens::TokenBuffer};

pub enum ParserError {
    EndOfFile,
}

fn constant(buffer: &mut TokenBuffer) -> Result<Expr, ParserError> {
    unimplemented!()
}
