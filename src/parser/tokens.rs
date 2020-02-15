use crate::{
    ast::{Ident, Literal},
    parser::{diag, prelude::*, PResult, Span, Token},
};

// Keywords
pub const FUNC: &'static str = "func";

pub fn literal(inp: Span) -> IResult<Span, PResult<Token<Literal>>> {
    alt((float, integer))(inp)
}

fn integer(inp: Span) -> IResult<Span, PResult<Token<Literal>>> {
    let sign = opt(one_of("+-"));

    map(recognize(preceded(sign, digit1)), |sp: Span| {
        let val = match sp.fragment.parse::<i64>() {
            Ok(v) => v,
            Err(_) => return Err(diag::known(diag::TOK0001)),
        };
        Ok(Token::new(sp, Literal::Integer(val)))
    })(inp)
}

fn float(inp: Span) -> IResult<Span, PResult<Token<Literal>>> {
    let exponent = preceded(one_of("eE"), preceded(opt(one_of("+-")), digit1));

    let after_decimal = preceded(char('.'), digit1);

    let int_part = preceded(opt(one_of("+-")), digit1);

    let number = pair(pair(int_part, after_decimal), opt(exponent));

    map(recognize(number), |sp: Span| {
        let val = match sp.fragment.parse::<f64>() {
            Ok(v) => v,
            Err(_) => return Err(diag::known(diag::TOK0002)),
        };
        Ok(Token::new(sp, Literal::Float(val)))
    })(inp)
}

fn ident(inp: Span) -> IResult<Span, PResult<Token<Ident>>> {
    let identifier = pair(
        one_of("_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        many1(one_of(
            "_0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        )),
    );
    map(recognize(identifier), |inp: Span| {
        Ok(Token::new(inp, Ident::new(inp.fragment)))
    })(inp)
}

#[cfg(test)]
mod tests {
    use super::*;

    single_token_test!(unsigned_int, literal("42") => Literal::Integer(42));
    single_token_test!(neg_int, literal("-42") => Literal::Integer(-42));
    single_token_test!(pos_int, literal("+42") => Literal::Integer(42));
    single_token_test!(too_large, literal("9223372036854776000") err diag::TOK0001);

    single_token_test!(unsigned_float, literal("4.2") => Literal::Float(4.2));
    single_token_test!(pos_float, literal("+4.2") => Literal::Float(4.2));
    single_token_test!(neg_float, literal("-4.2") => Literal::Float(-4.2));
    single_token_test!(float_exp, literal("4.2e2") => Literal::Float(4.2e2));
    single_token_test!(pos_float_exp, literal("+4.2e+2") => Literal::Float(4.2e2));
    single_token_test!(neg_float_exp, literal("-4.2e-2") => Literal::Float(-4.2e-2));

    single_token_test!(identifier, ident("_abc123") => Ident::new("_abc123"));
}
