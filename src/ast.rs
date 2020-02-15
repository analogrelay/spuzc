#[derive(Debug, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Constant(Literal),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Ident(String);

impl Ident {
    pub fn new<S: Into<String>>(val: S) -> Ident {
        Ident(val.into())
    }
}

pub enum Decl {
    Func {
        pub name: Ident,
        pub return_type: Ident,
    },
}

impl Decl {
    pub fn func(name: Ident, return_type: Ident) -> Decl {
        Decl::Func { name, return_type }
    }
}
