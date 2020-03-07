use crate::text::{Spanned, Window};
use crate::tokens::{Token, TokenError};

pub fn next_token(window: &mut Window) -> Result<Spanned<Token>, TokenError> {
    // Discard whitespace
    window.take_while(char::is_whitespace)?;
    window.advance();

    match window.take()? {
        '(' => Ok(window.complete(Token::LParen)),
        ')' => Ok(window.complete(Token::RParen)),
        ':' => Ok(window.complete(Token::Colon)),
        '{' => Ok(window.complete(Token::LBrace)),
        '}' => Ok(window.complete(Token::RBrace)),
        '0'..='9' => number(window),
        x if x.is_alphabetic() || x == '_' => ident(window),
        x => Err(TokenError::Unexpected(x)),
    }
}

fn number(window: &mut Window) -> Result<Spanned<Token>, TokenError> {
    window.take_while('0'..='9')?;
    let s = window.content().to_owned();
    match s.parse() {
        Ok(n) => Ok(window.complete(Token::Integer(n))),
        Err(_) => Err(TokenError::InvalidNumber(s)),
    }
}

fn ident(window: &mut Window) -> Result<Spanned<Token>, TokenError> {
    window.take_while(|c: char| c.is_alphanumeric() || c == '_')?;
    let tok = match window.content() {
        "func" => Token::Func,
        "int" => Token::Int,
        x => Token::Identifier(x.to_owned()),
    };
    Ok(window.complete(tok))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{Document, Window};

    macro_rules! complete_token_test {
        ($name: ident, $content: expr, $token: expr) => {
            #[test]
            pub fn $name() {
                let orig = $content;
                // Char count may not equal 'len' because of multi-byte UTF-8 encodings.
                let char_count = orig.chars().count();
                let content = format!("    {}    next", orig);
                let doc = Document::new(content);
                let mut win = Window::new(&doc);
                let tok = next_token(&mut win).unwrap();

                assert_eq!(doc.text_at(tok.span), orig);
                assert_eq!(tok.span.start, (4, 0, 4).into());
                assert_eq!(tok.span.end, (4 + orig.len(), 0, 4 + char_count).into());
                assert_eq!(tok.value, $token);
            }
        };
    }

    complete_token_test!(lparen, "(", Token::LParen);
    complete_token_test!(rparen, ")", Token::RParen);
    complete_token_test!(colon, ":", Token::Colon);
    complete_token_test!(lbrace, "{", Token::LBrace);
    complete_token_test!(rbrace, "}", Token::RBrace);
    complete_token_test!(ident_alpha, "ident", Token::Identifier("ident".into()));
    complete_token_test!(
        ident_alpha_num,
        "ident42",
        Token::Identifier("ident42".into())
    );
    complete_token_test!(ident_unicode, "京¾৬", Token::Identifier("京¾৬".into()));
    complete_token_test!(ident_underscore, "_¾৬", Token::Identifier("_¾৬".into()));

    complete_token_test!(unsigned_integer, "1234", Token::Integer(1234));

    complete_token_test!(keyword_func, "func", Token::Func);
    complete_token_test!(keyword_int, "int", Token::Int);
}
