#[derive(Debug)]
pub enum Error {
    ParseIntError(String),
    ParseFloatError(String),
}
