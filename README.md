
# Mergeable errors

Zero-cost error handling for generic traits.

## Rationale

Assume we want to build a crate that defines a generic trait,
meant to be implemented by others.
Below is a minimalist example of such a trait:

```rust
    pub trait Producer {
        fn produce(&self) -> u16;
    }
```

Some implementations might work well with this definitions,
but other implementations may encounter errors in some situations
(*e.g.* `IOError` for a file-system based implementation).
We call the former kind *infallible implementations*,
and the latter kind *fallible implementations*.

In order to support both kinds of implementations,
the methods of our trait should rather return `Result<_, _>`.
This raises the question of the error type that those results should contain.

An option is to define a dedicated error type for our crate,
and force implementers to wrap their errors into this type.

```rust
    pub trait Producer {
        fn produce(&self) -> Result<u16, MyError>;
    }
```

This works, but breaks the "zero-cost abstraction" motto for infallible implementations.
Indeed, `Result<T, MyError>` can be significantly bigger than the type `T` alone.
For example, with a simple `MyError` type defined with the
[error_chain] crate,

* `<Result<(), MyError>` is 56 bytes long (*versus* 0 bytes for `()`),
* `<Result<u16, MyError>` is 64 bytes long (*versus* 2 bytes for `u16`).

A more flexible option is to let implementers specify their own error type:

```rust
    pub trait Producer {
        type Error: Error + Send + 'static;
        fn produce(&self) -> Result<u16, Self::Error>;
    }
```

For infallible implementations,
the `Error` associated type can be set to [`never`]
or any other value-less type (typically an empty Enum).
The compiler will then optimize away this error-type from the `Result`,
effectively returning only the success-type.

We now have a real zero-cost abstraction,
where infallible implementations do not pay the toll of error handling.
On the other hand, it will be harder to work with several heterogeneous implementations of our trait.
Consider for example the following type:

```rust
    pub struct PMax<P1, P2> (P1, P2);

    impl<P1: Producer, P2: Producer> Producer for PMax<P1, P2> {
        type Error = ???; // <<<< we have a problem here
        fn produce(&self) -> Result<u16, Self::Error> { 
            Ok(self.0.produce()?.max(self.1.produce()?))
        }

    }
```

Since `P1` and `P2` may use totally unrelated error types,
we don't know which error type to use.
We could use a "chainable" error type as defined by [error_chain],
but then we would go back to using a "fat" result even when `P1` and `P2` are both infallible.

## The solution

This crate provides a solution to the problems described above.
The idea is:

* to provide zero-cost error handling for infallible implementations,
  by allowing them to use [`never`] as their error type;
* to limit heterogeneity among fallible implementations,
  by requiring them to use a dedicated error type defined by the trait designer;
* to let the compiler infer the best error type when combining several implementations.

The example above would become:

```rust
    /// a dedicated error type
    pub struct MyError { /* ... */ }

    // define appropriate types and traits
    mergeable_errors!(MyError);

    pub trait Producer {
        // require that Producer::Error be either never of MyError
        type Error: MergesWith<MyError>;
        fn produce(&self) -> Result<u16, Self::Error>;
    }

    pub struct PMax<P1, P2> (P1, P2);
    impl<P1: Producer, P2: Producer> Producer for PMax<P1, P2> 
        // this trait bound is required to be able to use MergedError below
        where P1::Error: MergesWith<P2::Error>
    {
        // compute the most appropriate Error type based on P1 and P2;
        // especially, if P1 and P2 are both infallible,
        // that PMax will be infallible as well.
        type Error = MergedError<P1::Error, P2::Error>;
        fn produce(&self) -> Result<u16, Self::Error> {
            Ok(
              // the merged error always implements From<_>
              // for both P1::Error and P2::Error,
              // so inner errors can simply be lifted with '?'
              self.0.produce()?
              .max(self.1.produce()?)
            )
        }

    }
```

The `mergeable_errors` macro takes care of defining the following traits and types:

* `MergesWith<E>` is a trait to let the compiler infer the correct merging of error types.
  The macro provides implementations so that [`never`] and [`never`] merge as [`never`], and that any other combination of [`never`] and `MyError` merge as `MyError`.
* `MergedError<E1, E2>` is a type alias using `MergesWith` to determine the appropriate merged error type.
* `MergedResult<T, E1, E2>` is a shortcut for  `Result<T, MergedError<E1, E2>>`.


### About [`never`]

Since the [`never`] type is currently unstable,
this crate actually defines its own version called `mergeable_errors::Never`.
Once [`never`] becomes stable,
`mergeable_errors::Never` will become a simple alias to [`never`],
avoiding a breaking change.


[error_chain]: https://docs.rs/error-chain/
[`never`]: https://doc.rust-lang.org/std/primitive.never.html
