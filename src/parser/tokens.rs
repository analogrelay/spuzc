use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::sequence::*;
use nom::IResult;

use crate::parser::{Error, Result, Span};

#[derive(Debug)]
pub struct Token<'a> {
    pub content: Span<'a>,
    pub kind: TokenKind,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Integer(i64),
    Float(f64),
}

impl<'a> Token<'a> {
    pub fn new(content: Span<'a>, kind: TokenKind) -> Token {
        Token { content, kind }
    }
}

fn integer(inp: Span) -> IResult<Span, Result<Token>> {
    let sign = opt(one_of("+-"));

    map(recognize(preceded(sign, digit1)), |sp: Span| {
        let val = match sp.fragment.parse::<i64>() {
            Ok(v) => v,
            Err(_) => return Err(Error::ParseIntError(sp.fragment.to_string())),
        };
        Ok(Token::new(sp, TokenKind::Integer(val)))
    })(inp)
}

fn float(inp: Span) -> IResult<Span, Result<Token>> {
    let after_decimal = preceded(char('.'), digit1);

    let exponent = preceded(one_of("eE"), pair(opt(one_of("+-")), digit1));

    let number = tuple((opt(one_of("+-")), digit1, opt(after_decimal), opt(exponent)));

    map(recognize(number), |sp: Span| {
        let val = match sp.fragment.parse::<f64>() {
            Ok(v) => v,
            Err(_) => return Err(Error::ParseFloatError(sp.fragment.to_string())),
        };
        Ok(Token::new(sp, TokenKind::Float(val)))
    })(inp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn can_recognize_digits() {
        let output = integer(Span::new("42")).unwrap();
        assert_eq!(
            output.0,
            Span {
                offset: 2,
                line: 1,
                fragment: "",
                extra: ()
            }
        );
        assert_eq!(output.1.unwrap().kind, TokenKind::Integer(42));
    }

    #[test]
    pub fn can_recognize_negative_digits() {
        let output = integer(Span::new("-42")).unwrap();
        assert_eq!(
            output.0,
            Span {
                offset: 3,
                line: 1,
                fragment: "",
                extra: ()
            }
        );
        assert_eq!(output.1.unwrap().kind, TokenKind::Integer(-42));
    }

    #[test]
    pub fn fails_if_too_large_for_i64() {
        assert_eq!(
            integer(Span::new("9223372036854776000")).unwrap_err(),
            nom::Err::Error((
                Span::new("9223372036854776000"),
                nom::error::ErrorKind::MapRes
            ))
        )
    }
}
