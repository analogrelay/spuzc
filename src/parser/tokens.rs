use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::sequence::*;
use nom::IResult;

use crate::parser::Span;

// pub struct Token {
//     pub content: Span,
//     pub kind: TokenKind,
// }

// pub enum TokenKind {
//     Integer(i64),
// }

fn signed_digits(inp: Span) -> IResult<Span, i64> {
    let sign = map(opt(alt((value(1, tag("+")), value(-1, tag("-"))))), |o| {
        o.unwrap_or(1) as i64
    });

    let digits = map_res(digit1, |s: Span| s.fragment.parse::<i64>());

    let parser = pair(sign, digits);

    map(parser, |(mult, digits)| digits * mult)(inp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn can_recognize_digits() {
        let output = signed_digits(Span::new("42")).unwrap();
        assert_eq!(
            output.0,
            Span {
                offset: 2,
                line: 1,
                fragment: "",
                extra: ()
            }
        );
        assert_eq!(output.1, 42);
    }

    #[test]
    pub fn can_recognize_negative_digits() {
        let output = signed_digits(Span::new("-42")).unwrap();
        assert_eq!(
            output.0,
            Span {
                offset: 3,
                line: 1,
                fragment: "",
                extra: ()
            }
        );
        assert_eq!(output.1, -42);
    }

    #[test]
    pub fn fails_if_too_large_for_i64() {
        assert_eq!(
            signed_digits(Span::new("9223372036854775808")),
            Err(nom::Err::Error((
                Span::new("9223372036854775808"),
                nom::error::ErrorKind::MapRes
            )))
        )
    }
}
