//! This module contains the traits and types defined by `mergeable_errors!`.

/// A dummy error type to demonstrate `mergeable_errors`.
#[derive(Debug)]
pub struct MyError {}
impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for MyError {}

mergeable_errors!(MyError);
