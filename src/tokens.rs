use crate::text::{Spanned, Window};

#[derive(Debug)]
pub enum TokenError {
    EndOfFile,
    Unexpected(char),
}

#[derive(Debug, PartialEq)]
pub enum Token {
    LParen,
    Integer(i128),
}

pub fn next_token(window: &mut Window) -> Result<Spanned<Token>, TokenError> {
    if let Some(c) = window.next() {
        match c {
            x => Err(TokenError::Unexpected(x)),
        }
    } else {
        Err(TokenError::EndOfFile)
    }
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
                let content = format!("    {}    next", orig);
                let doc = Document::new(content);
                let mut win = Window::new(&doc);
                let tok = next_token(&mut win).unwrap();

                assert_eq!(doc.text_at(tok.span), orig);
                assert_eq!(tok.span.start.tup(), (4, 0, 4));
                assert_eq!(tok.span.end.tup(), (4 + orig.len(), 0, 4 + orig.len()));
                assert_eq!(tok.value, $token);
            }
        };
    }

    complete_token_test!(lparen, "(", Token::LParen);
}
