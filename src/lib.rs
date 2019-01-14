#[macro_export]
macro_rules! mergeable_errors {
    () => {
        mergeable_errors!(Error);
    };
    ($error: ty) => {
        mergeable_errors!(Error, Never, OkResult, MergesWith, MergedResult);
    };
    ($error: ty, $never: ident, $ok_result: ident, $merges_with: ident, $merged_result: ident) => {
        /// An "error" type that can never happen.
        ///
        /// NB: once the [`never`] types reaches *stable*,
        /// this type will be an alias for the standard type.
        ///
        /// [`never`]: https://doc.rust-lang.org/std/primitive.never.html
        ///
        #[derive(Clone, Debug)]
        pub enum $never {}
        impl ::std::fmt::Display for $never {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "$never")
            }
        }
        impl std::error::Error for $never {}
        // This conversion can $never happen (since $never can have no value),
        // but it is required for using `?` with `$ok_results`s.
        impl From<$never> for $error {
            fn from(_: $never) -> $error { unreachable!() }
        }
        /// Type alias for a result that will $never fail.
        pub type $ok_result<T> = std::result::Result<T, $never>;


        /// A utility trait for merging two types of errors.
        pub trait $merges_with<E>: Sized + std::marker::Send + std::error::Error + 'static {
            type Into: std::marker::Send + std::error::Error + 'static + From<Self> + From<E>;
        }
        impl<T: std::marker::Send + std::error::Error + 'static> $merges_with<$error> for T where $error: From<T> { type Into = $error; }
        impl<T: std::marker::Send + std::error::Error + 'static> $merges_with<$never> for T where T: From<$never> { type Into = T; }


        /// A shortcut for building the merged result type,
        /// given one value type and two error types,
        /// which must both be either `$error` or [`$never`].
        /// 
        /// [`$never`]: enum.$never.html
        pub type $merged_result<T, E1, E2> = std::result::Result<T, <E1 as $merges_with<E2>>::Into>;
    };
}
