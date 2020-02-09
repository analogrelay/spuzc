use nom_locate::LocatedSpan;

mod error;
mod tokens;

pub use error::Error;

pub type Span<'a> = LocatedSpan<&'a str>;
pub type Result<T> = ::std::result::Result<T, Error>;
