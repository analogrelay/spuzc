use nom_locate::LocatedSpan;

pub mod diag;
mod tokens;

pub use diag::Diagnostic;

pub type Span<'a> = LocatedSpan<&'a str>;
pub type Result<T> = ::std::result::Result<T, Vec<Diagnostic>>;
