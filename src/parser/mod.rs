use nom_locate::LocatedSpan;

#[macro_use]
mod macros;

mod decls;
pub mod diag;
mod exprs;
mod prelude;
mod tokens;

pub use diag::Diagnostic;

pub type Span<'a> = LocatedSpan<&'a str>;
pub type PResult<T> = ::std::result::Result<T, Vec<Diagnostic>>;

#[derive(Debug)]
pub struct Token<'a, T> {
    pub content: Span<'a>,
    pub value: T,
}

impl<'a, T> Token<'a, T> {
    pub fn new(content: Span<'a>, value: T) -> Token<T> {
        Token {
            content,
            value: value,
        }
    }

    pub fn map<F, U>(self, wrapper: F) -> Token<'a, U>
    where
        F: Fn(T) -> U,
    {
        Token::new(self.content, wrapper(self.value))
    }
}
