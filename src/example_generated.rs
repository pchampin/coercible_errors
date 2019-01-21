//! This module contains the traits and types defined by `coercible_errors!`.

/// A dummy error type to demonstrate `coercible_errors`.
#[derive(Debug)]
pub struct MyError {}
impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for MyError {}

coercible_errors!(MyError);
