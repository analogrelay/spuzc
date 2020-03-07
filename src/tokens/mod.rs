use crate::{
    ast::Token,
    text::{Span, Spanned, TextError, Window},
};

mod rules;

#[derive(Debug)]
pub enum TokenError {
    EndOfFile,
    NoCurrentToken,
    InvalidText(TextError),
    InvalidNumber(String),
    Unexpected(char),
}

impl From<TextError> for TokenError {
    fn from(e: TextError) -> TokenError {
        match e {
            TextError::OutOfBounds => TokenError::EndOfFile,
            e => TokenError::InvalidText(e),
        }
    }
}

pub struct TokenBuffer<'a> {
    window: Window<'a>,
    span: Span,
    content: Vec<Token>,
    buffer: Option<Spanned<Token>>,
}

impl<'a> TokenBuffer<'a> {
    pub fn new(window: Window<'a>) -> TokenBuffer<'a> {
        TokenBuffer {
            window,
            span: Span::ZERO,
            content: Vec::new(),
            buffer: None,
        }
    }

    pub fn complete(&mut self) -> Spanned<Vec<Token>> {
        let new_end = Span::new(self.span.end, self.span.end);
        Spanned::new(
            std::mem::replace(&mut self.content, Vec::new()),
            std::mem::replace(&mut self.span, new_end),
        )
    }

    pub fn peek(&mut self) -> Result<&Spanned<Token>, TokenError> {
        if self.buffer.is_none() {
            let token = match rules::next_token(&mut self.window) {
                Ok(spanned) => spanned,
                Err(e) => return Err(e),
            };
            self.buffer = Some(token)
        }
        self.buffer.as_ref().ok_or(TokenError::EndOfFile)
    }

    pub fn take(&mut self) -> Result<(), TokenError> {
        self.peek()?;

        match self.buffer.take() {
            Some(s) => {
                self.span = self
                    .span
                    .append(s.span)
                    .expect("Window returned non-contiguous span!");
                self.content.push(s.value);
                Ok(())
            }
            None => Err(TokenError::NoCurrentToken),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{Document, Window};

    #[test]
    pub fn token_buffer_peek() {
        let content = "(42)";
        let doc = Document::new(content);
        let win = Window::new(&doc);
        let mut buf = TokenBuffer::new(win);

        assert_eq!(
            buf.peek().unwrap(),
            &Spanned::new(Token::LParen, Span::new((0, 0, 0).into(), (1, 0, 1).into()))
        );
        assert_eq!(
            buf.peek().unwrap(),
            &Spanned::new(Token::LParen, Span::new((0, 0, 0).into(), (1, 0, 1).into()))
        );
        assert_eq!(
            buf.peek().unwrap(),
            &Spanned::new(Token::LParen, Span::new((0, 0, 0).into(), (1, 0, 1).into()))
        );
    }

    #[test]
    pub fn token_buffer_take() {
        let content = "(42)";
        let doc = Document::new(content);
        let win = Window::new(&doc);
        let mut buf = TokenBuffer::new(win);

        buf.take().unwrap();
        assert_eq!(
            buf.peek().unwrap(),
            &Spanned::new(
                Token::Integer(42),
                Span::new((1, 0, 1).into(), (3, 0, 3).into())
            )
        )
    }

    #[test]
    pub fn token_buffer_complete() {
        let content = "(42)";
        let doc = Document::new(content);
        let win = Window::new(&doc);
        let mut buf = TokenBuffer::new(win);

        assert_eq!(buf.complete(), Spanned::new(vec![], Span::ZERO));
        buf.take().unwrap();
        buf.take().unwrap();
        assert_eq!(
            buf.complete(),
            Spanned::new(
                vec![Token::LParen, Token::Integer(42)],
                Span::new((0, 0, 0).into(), (3, 0, 3).into())
            )
        );
        assert_eq!(
            buf.peek().unwrap(),
            &Spanned::new(Token::RParen, Span::new((3, 0, 3).into(), (4, 0, 4).into()))
        );
    }
}
