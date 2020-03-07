// Disable dead_code warnings on debug builds, while we're developing things.
#![cfg_attr(debug_assertions, allow(dead_code))]

mod ast;
mod parser;
mod text;
mod tokens;
