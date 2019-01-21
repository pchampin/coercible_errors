mod error {
    error_chain! {
        errors {
            Producer {
                description("error occuredn in producer"),
            }
            Consumer {
                description("error occurent in consumer"),
            }
        }
    }
    coercible_errors! {}

}

use self::error::*;
use std::result::Result as StdResult;

pub trait Producer {
    /// The error type that this producer can raise.
    /// Must be either [`Error`] or [`Never`].
    ///
    /// [`Error`]: struct.Error.html
    /// [`Never`]: enum.Never.html
    type Error: CoercibleWith<Error>;

    /// Produce a value.
    fn produce(&self) -> StdResult<u16, Self::Error>;
}

impl Producer for u16 {
    type Error = Never;
    fn produce(&self) -> StdResult<u16, Self::Error> {
        Ok(*self)
    }
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
    type Error: CoercibleWith<Error>;
    fn consume(&mut self, val: u16) -> StdResult<(), Self::Error>;
}

impl Consumer for u16 {
    /// The error type that this producer can raise.
    /// Must be either [`Error`] or [`Never`].
    ///
    /// [`Error`]: struct.Error.html
    /// [`Never`]: enum.Never.html
    type Error = Never;

    /// Consume a value.
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

pub struct PMax<P1, P2>(P1, P2);

impl<P1: Producer, P2: Producer> Producer for PMax<P1, P2>
where
    P1::Error: CoercibleWith<P2::Error>,
{
    type Error = CoercedError<P1::Error, P2::Error>;
    fn produce(&self) -> CoercedResult<u16, P1::Error, P2::Error> {
        Ok(self.0.produce()?.max(self.1.produce()?))
    }
}

/// This is the naive version of pipe;
/// it always returns a `Result`,
/// even if both the producer and the consumer return `OkResult`s.
fn pipe1<P: Producer, C: Consumer>(p: &P, c: &mut C) -> Result<()> {
    let v = p.produce().chain_err(|| ErrorKind::Producer)?;
    c.consume(v).chain_err(|| ErrorKind::Consumer)
}

/// This is the smart version of pipe;
/// it returns the smallest possible result (`Result` or `OkResult`),
/// based on what the producer and the consumer return.
fn pipe2<P: Producer, C: Consumer>(p: &P, c: &mut C) -> CoercedResult<(), P::Error, C::Error>
where
    P::Error: CoercibleWith<C::Error>,
{
    Ok(c.consume(p.produce()?)?)
}

#[test]
fn test() -> Result<()> {
    // NB: most of this test is actually performed at compile time:
    // we check that the coerced result types are as expected
    // (Result<_> or OkResult<_>).

    let mut cons8: u8 = 0;
    let mut cons16: u16 = 0;

    // ########## pipe1 ##########
    // pipe1 always chain inner errors into Error
    let _r: Result<()> = pipe1(&42_u16, &mut cons16);
    let _r: Result<()> = pipe1(&42_u16, &mut cons8);
    let _r: Result<()> = pipe1(&42_u32, &mut cons16);
    let _r: Result<()> = pipe1(&42_u32, &mut cons8);

    let _r: Result<()> = pipe1(&0x20000_u32, &mut cons8);
    let _r: Result<()> = pipe1(&0x200_u16, &mut cons8);

    // this is already a good thing,
    // because only methods needing to *coerce* errors
    // need to fallback to Error;
    // simple methods may still use Self::Error, e.g.:
    let _r: OkResult<u16> = 42_u16.produce();
    let _r: OkResult<()>  = cons16.consume(42);
    let _r: Result<u16>   = 42_u32.produce();
    let _r: Result<()>    = cons8.consume(42);

    // ########## pipe2 ##########
    // pipe2 infers the minimal type from its arguments
    let _r: OkResult<()> = pipe2(&42_u16, &mut cons16);
    let _r: Result<()>   = pipe2(&42_u16, &mut cons8);
    let _r: Result<()>   = pipe2(&42_u32, &mut cons16);
    let _r: Result<()>   = pipe2(&42_u32, &mut cons8);

    let _r: Result<()>   = pipe2(&0x20000_u32, &mut cons8);
    let _r: Result<()>   = pipe2(&0x200_u16, &mut cons8);

    // ######## PMax ########
    let _r: OkResult<u16> = PMax(42_u16, 1_u16).produce();
    let _r: Result<u16>   = PMax(42_u32, 1_u16).produce();
    let _r: Result<u16>   = PMax(42_u16, 1_u32).produce();
    let _r: Result<u16>   = PMax(42_u32, 1_u32).produce();

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
    let r6: Result<()> = pipe1(&0x200_u16, &mut cons8);
    //println!("{:?}\n{:?}", r5, r6);
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
    let r6: Result<()> = pipe2(&0x200_u16, &mut cons8);
    //println!("{:?}\n{:?}", r5, r6);
    assert!(r5.is_err());
    assert!(r6.is_err());

    let r1: OkResult<u16> = PMax(42_u16, 1_u16).produce();
    let r2: Result<u16>   = PMax(42_u32, 1_u16).produce();
    let r3: Result<u16>   = PMax(42_u16, 1_u32).produce();
    let r4: Result<u16>   = PMax(42_u32, 1_u32).produce();
    //println!("{:?} {:?} {:?} {:?}", r1, r2, r3, r4);
    assert!(r1.unwrap() == 42);
    assert!(r2.unwrap() == 42);
    assert!(r3.unwrap() == 42);
    assert!(r4.unwrap() == 42);

    println!("All tests passed",);

    Ok(())
}
