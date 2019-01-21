//! Zero-cost error handling for generic traits.
//!
//! # Rationale
//!
//! Assume you want to build a crate that defines a generic trait,
//! meant to be implemented by others.
//! Some implementations of that trait will always succeed,
//! others may sometimes fail.
//! The methods of the generic trait should therefore return `Result<_,_>`,
//! but that should not induce an overhead for infallible implementations
//! (per the *zero-cost abstraction* motto).
//!
//! The [`coercible_errors!`] macro will define a set of traits and types
//! that you can use to define your generic traits,
//! in order to keep their error-handling as efficient as possible.
//! More precisely, the compiler will be able to optimize away the error types
//! whenever only infallible implementations of the generic trait are used.
//!
//! See `README.md` for a more detailed explaination.
//!
//! [`coercible_errors!`]: macro.coercible_errors.html

/// Sets up coercible_errors for a previously defined error type.
///
/// It re-exports the types [`Never`] and [`OkResult`],
/// and defines three new traits and types `CoercibleWith`,
/// `CoercedError` and `CoercedResult`.
///
/// [`Never`]: enum.Never.html
/// [`OkResult`]: type.OkResult.html
#[macro_export]
macro_rules! coercible_errors {
    () => {
        coercible_errors!(Error);
    };
    ($error: ty) => {
        coercible_errors!($error, CoercibleWith, CoercedError, CoercedResult);
    };
    ($error: ty, $coercible_with: ident, $coerced_error: ident, $coerced_result: ident) => {
        pub use $crate::{Never, OkResult};

        // This conversion can never happen (since Never can have no value),
        // but it is required for allowing $error and Never to coerce with each other.
        impl From<Never> for $error {
            fn from(_: Never) -> $error {
                unreachable!()
            }
        }

        /// A trait used to determine how to best coerce two error types.
        ///
        /// In practice, the only two error types that it handles are `$error` or `Never`.
        pub trait $coercible_with<E>:
            Sized + std::marker::Send + std::error::Error + 'static
        {
            type Into: std::marker::Send
                + std::error::Error
                + 'static
                + From<Self>
                + From<E>
                + $coercible_with<$error>;
        }
        impl $coercible_with<$error> for $error {
            type Into = $error;
        }
        impl $coercible_with<Never> for $error {
            type Into = $error;
        }
        impl $coercible_with<$error> for Never {
            type Into = $error;
        }
        impl $coercible_with<Never> for Never {
            type Into = Never;
        }

        /// A shortcut for building the coerced error type,
        /// given two error types,
        /// which must both be either `$error` or `Never`.
        pub type $coerced_error<E1, E2> = <E1 as $coercible_with<E2>>::Into;

        /// A shortcut for building the coerced result type,
        /// given one value type and two error types,
        /// which must both be either `$error` or `Never`.
        pub type $coerced_result<T, E1, E2> = std::result::Result<T, $coerced_error<E1, E2>>;
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
