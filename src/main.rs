//! This crate is an exploration for building
//! a zero-cost error-handling for generic type.
//! 
//! The idea is that:
//! the methods of a generic type should return Results,
//! to account for implementation that may fail.
//! But *some* implementations may never fail,
//! and those would be penalized
//! by the overhead of wrapping their returned values into Results.
//! 
//! To avoid that, we define a trait MaybeError,
//! which is implemented by a crate Error type,
//! and by an empty enum Never (which can be cast to Error).
//! 
#[macro_use] extern crate error_chain;

use std::error::Error as StdError;
use std::result::Result as StdResult;

mod error {
    error_chain!{
        errors {
            InvalidIri(iri: String) {
                description("invalid iri"),
                display("invalid iri <{}>", iri),
            }
            InvalidLanguageTag(lang: String) {
                description("invalid language tag"),
                display("invalid language tag \"{}\"", lang),
            }
            OtherError {
                description("other error"),
            }
            GraphError {
                description("error while quering graph"),
            }
            GraphMutationError {
                description("error while modifying graph"),
            }
        }
    }
}

use self::error::*;

#[derive(Clone, Debug)]
pub enum Never {}
impl ::std::fmt::Display for Never {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "Never")
    }
}
impl StdError for Never {}

// This conversion can never happen (since Never can have no value),
// but it is required for using `?` with `OkResult`s.
impl From<Never> for Error {
    fn from(_: Never) -> Error { unreachable!() }
}
impl From<Error> for Never {
    fn from(_: Error) -> Never { unreachable!() }
}

pub type OkResult<T> = StdResult<T, Never>;



/// A supertype of [`Error`] and [`Never`].
/// 
/// NB: this trait *must not* be implemented by any other type.
/// 
/// [`Error`]: struct.Error.html
/// [`Never`]: enum.Never.html
pub trait MaybeError: Into<Error> + Into<Never> + StdError + 'static {}
impl MaybeError for Error {}
impl MaybeError for Never {}

/// A utility trait for merging two types of [`MaybeError`](trait.MaybeError.html).
/// 
/// It will generally not be used directly,
/// but through [`Merge`](type.Merge.html)
pub trait MaybeErrorPair<E1, E2> { type Super: MaybeError + From<E1> + From<E2>; }
impl MaybeErrorPair<Error, Error> for () { type Super = Error; }
impl MaybeErrorPair<Error, Never> for () { type Super = Error; }
impl MaybeErrorPair<Never, Error> for () { type Super = Error; }
impl MaybeErrorPair<Never, Never> for () { type Super = Never; }

/// A shortcut for determining the most general supertype of `E1` and `E2`,
/// which must both be either [`Error`] or [`Never`].resiter
/// 
/// [`Error`]: struct.Error.html
/// [`Never`]: enum.Never.html
pub type Merge<T, E1, E2> = StdResult<T, <() as MaybeErrorPair<E1, E2>>::Super>;

pub trait MaybeResultExt<T> {
    fn lift(self) -> Result<T>;
}

impl<T, E: MaybeError> MaybeResultExt<T> for StdResult<T, E> {
    fn lift(self) -> Result<T> {
        match self {
            Ok(v) => Ok(v),
            Err(maybe) => Err(maybe.into()),
        }
    }
}

pub trait MaybeResultExt2<T, E1, E2> {
    fn lift2(self) -> Merge<T, E1, E2> where
        (): MaybeErrorPair<E1, E2>
    ;
}

impl<T> MaybeResultExt2<T, Error, Error> for StdResult<T, Error> {
    fn lift2(self) -> Merge<T, Error, Error> { self }
}
impl<T> MaybeResultExt2<T, Never, Error> for StdResult<T, Error> {
    fn lift2(self) -> Merge<T, Never, Error> { self }
}
impl<T> MaybeResultExt2<T, Error, Never> for StdResult<T, Error> {
    fn lift2(self) -> Merge<T, Error, Never> { self }
}
impl<T> MaybeResultExt2<T, Error, Never> for StdResult<T, Never> {
    fn lift2(self) -> Merge<T, Error, Never> { self.lift() }
}
impl<T> MaybeResultExt2<T, Never, Error> for StdResult<T, Never> {
    fn lift2(self) -> Merge<T, Never, Error> { self.lift() }
}
impl<T> MaybeResultExt2<T, Never, Never> for StdResult<T, Never> {
    fn lift2(self) -> Merge<T, Never, Never> { self }
}


pub trait Producer {
    type Error: MaybeError;
    fn produce(&self) -> StdResult<u16, Self::Error>;
}

impl Producer for u16 {
    type Error = Never;
    fn produce(&self) -> StdResult<u16, Self::Error> { Ok(*self) }
}

impl Producer for u32 {
    type Error = Error;
    fn produce(&self) -> StdResult<u16, Self::Error> {
        if *self <= 0xFFFF {
            Ok(*self as u16)
        } else {
            bail!("Value too big to be produced")
        }
    }
}

pub trait Consumer {
    type Error: MaybeError;
    fn consume(&mut self, val: u16) -> StdResult<(), Self::Error>;
}

impl Consumer for u16 {
    type Error = Never;
    fn consume(&mut self, val: u16) -> StdResult<(), Self::Error> {
        *self = val;
        Ok(())
    }
}

impl Consumer for u8 {
    type Error = Error;
    fn consume(&mut self, val: u16) -> StdResult<(), Self::Error> {
        if val <= 0xff {
            *self = val as u8;
            Ok(())
        } else {
            bail!("Value too big to be consumed")
        }
    }
}

fn pipe1<P: Producer, C: Consumer>(p: &P, c: &mut C)-> Result<()> {
    c.consume(p.produce().lift()?).lift()
}

fn pipe2<P: Producer, C: Consumer>(p: &P, c: &mut C)
    -> Merge<(), P::Error, C::Error>
where
    (): MaybeErrorPair<P::Error, C::Error>,
    StdResult<u16, P::Error>: MaybeResultExt2<u16, P::Error, C::Error>,
    StdResult<(),  C::Error>: MaybeResultExt2<(),  P::Error, C::Error>,
{
    c.consume(p.produce().lift2()?).lift2()
    //unimplemented!()
}

fn main() -> Result<()> {
    println!("Result<u16>: {}", std::mem::size_of::<Result<u8>>());
    println!("Ok<u16>: {}", std::mem::size_of::<OkResult<u8>>());
    println!("Ok<()>: {}", std::mem::size_of::<OkResult<()>>());

    let mut cons8: u8 = 0;
    let mut cons16: u16 = 0;

    // ########## pipe1 ##########
    // pipe1 always lifts MaybeError into Error
    let _r: Result<()> = pipe1(&42_u16, &mut cons16);
    let _r: Result<()> = pipe1(&42_u16, &mut cons8);
    let _r: Result<()> = pipe1(&42_u32, &mut cons16);
    let _r: Result<()> = pipe1(&42_u32, &mut cons8);

    let _r: Result<()> = pipe1(&0x20000_u32, &mut cons8);
    let _r: Result<()> = pipe1(&0x200_u16,   &mut cons8);

    // this is already a good thing,
    // because only methods needing to *merge* errors
    // need to fallback to Error;
    // simple methods may still use Self::Error, e.g.:
    let _r: OkResult<u16> = 42_u16.produce();
    let _r: OkResult<()> = cons16.consume(42);
    let _r: Result<u16> = 42_u32.produce();
    let _r: Result<()> = cons8.consume(42);

    // ########## pipe1 ##########
    // pipe2 infers the minimal type from its arguments
    let _r: OkResult<()> = pipe2(&42_u16, &mut cons16);
    let _r: Result<()>   = pipe2(&42_u16, &mut cons8);
    let _r: Result<()>   = pipe2(&42_u32, &mut cons16);
    let _r: Result<()>   = pipe2(&42_u32, &mut cons8);


    let _r: Result<()>   = pipe2(&0x20000_u32, &mut cons8);
    let _r: Result<()>   = pipe2(&0x200_u16,   &mut cons8);

    // ######## testing the returned values ########
    // (having the correct type is not enough...)

    let r1: Result<()> = pipe1(&42_u16, &mut cons16);
    let r2: Result<()> = pipe1(&42_u16, &mut cons8);
    let r3: Result<()> = pipe1(&42_u32, &mut cons16);
    let r4: Result<()> = pipe1(&42_u32, &mut cons8);
    //println!("{:?} {:?} {:?} {:?}", r1, r2, r3, r4);
    assert!(r1.is_ok());
    assert!(r2.is_ok());
    assert!(r3.is_ok());
    assert!(r4.is_ok());

    let r5: Result<()> = pipe1(&0x20000_u32, &mut cons8);
    let r6: Result<()> = pipe1(&0x200_u16,   &mut cons8);
    println!("{:?}\n{:?}\n", r5, r6);
    assert!(r5.is_err());
    assert!(r6.is_err());

    let r1: OkResult<()> = pipe2(&42_u16, &mut cons16);
    let r2: Result<()>   = pipe2(&42_u16, &mut cons8);
    let r3: Result<()>   = pipe2(&42_u32, &mut cons16);
    let r4: Result<()>   = pipe2(&42_u32, &mut cons8);
    //println!("{:?} {:?} {:?} {:?}", r1, r2, r3, r4);
    assert!(r1.is_ok());
    assert!(r2.is_ok());
    assert!(r3.is_ok());
    assert!(r4.is_ok());

    let r5: Result<()> = pipe2(&0x20000_u32, &mut cons8);
    let r6: Result<()> = pipe2(&0x200_u16,   &mut cons8);
    println!("{:?}\n{:?}\n", r5, r6);
    assert!(r5.is_err());
    assert!(r6.is_err());

    Ok(())
}
