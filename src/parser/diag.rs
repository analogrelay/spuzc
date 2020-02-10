use std::borrow::Cow;

pub const TOK0001: (&'static str, &'static str) = ("TOK0001", "Failed to parse as integer.");
pub const TOK0002: (&'static str, &'static str) = ("TOK0002", "Failed to parse as floating-point.");

#[derive(Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub id: &'static str,
    pub message: Cow<'static, str>,
}

pub fn known(diag: (&'static str, &'static str)) -> Vec<Diagnostic> {
    let (id, message) = diag;
    vec![Diagnostic {
        id,
        message: Cow::from(message),
    }]
}
