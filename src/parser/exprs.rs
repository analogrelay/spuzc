use crate::{
    ast::{Expr, Literal},
    parser::{prelude::*, tokens, PResult, Span, Token},
};

pub fn expr(inp: Span) -> IResult<Span, PResult<Token<Expr>>> {
    map(tokens::literal, |inp: PResult<Token<Literal>>| match inp {
        Ok(t) => Ok(t.map(|v| Expr::Constant(v))),
        Err(e) => Err(e),
    })(inp)
}

#[cfg(test)]
mod tests {
    use super::*;

    single_token_test!(const_expr, expr("42") => Expr::Constant(Literal::Integer(42)));
}
