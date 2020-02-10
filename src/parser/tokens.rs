use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::sequence::*;
use nom::IResult;

use crate::parser::{diag, Result, Span};

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
            Err(_) => return Err(diag::known(diag::TOK0001)),
        };
        Ok(Token::new(sp, TokenKind::Integer(val)))
    })(inp)
}

fn float(inp: Span) -> IResult<Span, Result<Token>> {
    let exponent = preceded(one_of("eE"), preceded(opt(one_of("+-")), digit1));

    let after_decimal = preceded(char('.'), digit1);

    let int_part = preceded(opt(one_of("+-")), digit1);

    let number = pair(pair(int_part, opt(after_decimal)), opt(exponent));

    map(recognize(number), |sp: Span| {
        let val = match sp.fragment.parse::<f64>() {
            Ok(v) => v,
            Err(_) => return Err(diag::known(diag::TOK0002)),
        };
        Ok(Token::new(sp, TokenKind::Float(val)))
    })(inp)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! single_token_test {
        ($name: ident, $parser: ident($content: expr) => $kind: expr) => {
            #[test]
            pub fn $name() {
                let content = format!("{} other_content", $content);
                let output = $parser(Span::new(&content)).unwrap();
                assert_eq!(
                    output.0,
                    Span {
                        offset: $content.len(),
                        line: 1,
                        fragment: " other_content",
                        extra: ()
                    }
                );
                assert_eq!(output.1.unwrap().kind, $kind);
            }
        };
        ($name: ident, $parser: ident($content: expr) err $known_diag: expr) => {
            #[test]
            pub fn $name() {
                let content = format!("{} other_content", $content);
                let diags = $parser(Span::new(&content)).unwrap().1.unwrap_err();
                assert_eq!(diags, diag::known($known_diag));
            }
        };
    }

    single_token_test!(unsigned_int, integer("42") => TokenKind::Integer(42));
    single_token_test!(neg_int, integer("-42") => TokenKind::Integer(-42));
    single_token_test!(pos_int, integer("+42") => TokenKind::Integer(42));
    single_token_test!(too_large, integer("9223372036854776000") err diag::TOK0001);

    single_token_test!(unsigned_float, float("4.2") => TokenKind::Float(4.2));
    single_token_test!(pos_float, float("+4.2") => TokenKind::Float(4.2));
    single_token_test!(neg_float, float("-4.2") => TokenKind::Float(-4.2));
    single_token_test!(float_exp, float("4.2e2") => TokenKind::Float(4.2e2));
    single_token_test!(pos_float_exp, float("+4.2e+2") => TokenKind::Float(4.2e2));
    single_token_test!(neg_float_exp, float("-4.2e-2") => TokenKind::Float(-4.2e-2));
}
