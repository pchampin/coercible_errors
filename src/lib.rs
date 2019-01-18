//! Zero-cost error handling for generic traits.
//!
//! # Rationale
//!
//! Assume we want to build a crate that defines a generic trait,
//! meant to be implemented by others.
//! Some implementations of that trait may always succeed,
//! others may sometimes fail.
//! The methods of of the generic trait should therefore return `Result<_,_>`,
//! but do not want that to create an overhead for infallible implementations
//! (per the *zero-cose abstraction* motto).
//!
//! See `README.md` for a more detailed explaination.

/// Sets up mergeable_errors for a previously defined error type.
///
/// It re-exports the types [`Never`] and [`OkResults`],
/// and defines three new traits and types `MergesWith`,
/// `MergedError` and `MergedResult`.
///
/// [`Never`]: enum.Never.html
/// [`OkResult`]: type.OkResult.html
#[macro_export]
macro_rules! mergeable_errors {
    () => {
        mergeable_errors!(Error);
    };
    ($error: ty) => {
        mergeable_errors!($error, MergesWith, MergedError, MergedResult);
    };
    ($error: ty, $merges_with: ident, $merged_error: ident, $merged_result: ident) => {
        pub use $crate::{Never, OkResult};

        // This conversion can never happen (since Never can have no value),
        // but it is required for allowing $error and Never to merge with each other.
        impl From<Never> for $error {
            fn from(_: Never) -> $error {
                unreachable!()
            }
        }

        /// A trait used to determine how to best merge two error types.
        ///
        /// In practice, the only two error types that it handles are `$error` or `Never`.
        pub trait $merges_with<E>: Sized + std::marker::Send + std::error::Error + 'static {
            type Into: std::marker::Send
                + std::error::Error
                + 'static
                + From<Self>
                + From<E>
                + MergesWith<$error>;
        }
        impl $merges_with<$error> for $error {
            type Into = $error;
        }
        impl $merges_with<Never> for $error {
            type Into = $error;
        }
        impl $merges_with<$error> for Never {
            type Into = $error;
        }
        impl $merges_with<Never> for Never {
            type Into = Never;
        }

        /// A shortcut for building the merged error type,
        /// given two error types,
        /// which must both be either `$error` or `Never`.
        pub type $merged_error<E1, E2> = <E1 as $merges_with<E2>>::Into;

        /// A shortcut for building the merged result type,
        /// given one value type and two error types,
        /// which must both be either `$error` or `Never`.
        pub type $merged_result<T, E1, E2> = std::result::Result<T, $merged_error<E1, E2>>;
    };
}

/// An "error" type that can never happen.
///
/// NB: once the [`never`] types reaches *stable*,
/// this type will be an alias for the standard type.
///
/// [`never`]: https://doc.rust-lang.org/std/primitive.never.html
///
#[derive(Clone, Debug)]
pub enum Never {}
impl ::std::fmt::Display for Never {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "Never")
    }
}
impl std::error::Error for Never {}

/// Type alias for a result that will Never fail.
pub type OkResult<T> = std::result::Result<T, Never>;

#[cfg(feature = "example_generated")]
pub mod example_generated;

#[cfg(test)]
#[macro_use]
extern crate error_chain;

#[cfg(test)]
mod test;
