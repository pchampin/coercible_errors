/// A dummy error type to demonstrate `mergeable_errors`.
#[derive(Debug)]
pub struct Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}

mergeable_errors!(Error);