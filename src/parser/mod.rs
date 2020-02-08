use nom_locate::LocatedSpan;

mod tokens;

pub type Span<'a> = LocatedSpan<&'a str>;
