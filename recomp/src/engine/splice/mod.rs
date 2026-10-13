//! WP-F: Splice VM (Sly 2's Scheme-like script language) + the 169 built-ins.
//! See ENGINE_PORT.md findings prefixed "F".

pub mod bif;
pub mod eval;
pub mod frame;
pub mod host;
pub mod serialize;
pub mod value;

pub use eval::Vm;
pub use host::{NullHost, SpliceHost};
pub use value::Ref;

#[derive(Debug, Clone, PartialEq)]
pub enum SpliceError {
    /// A world-touching BIF/method the host has not implemented yet.
    HostUnimplemented(String),
    BadBif(u32),
    NotCallable(i32),
    Arity,
    Type(&'static str, i32),
    Malformed(&'static str),
    DivideByZero,
    Read(String),
}

impl std::fmt::Display for SpliceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SpliceError {}
