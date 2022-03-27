//! Parsers for numeric literals.
use num_traits::{CheckedAdd, CheckedMul, CheckedNeg, Zero};
use somen::prelude::*;

use crate::Character;

/// Generates a function returns a integer parser.
///
/// This macro takes comma-separated list of patterns `prefix => radix` like `"0x" => 16`, and `_ => radix`
/// in the last are interpreted as a fallback without prefix.
///
/// # Examples
/// ```
/// # futures::executor::block_on(async {
/// use somen::prelude::*;
/// use somen_language::numeric::signed;
/// use somen_language::int_parser;
///
/// let mut parser = signed::<i64, _, _, _, _>(
///     int_parser!{
///         "0x" => 16,
///         "0o" => 8,
///         "0b" => 2,
///         _ => 10,
///     },
///     true,
/// ).skip(eof());
///
/// let mut src1 = stream::from_iter("0xDEADb33f".chars())
///     .positioned::<usize>()
///     .buffered_rewind();
/// assert_eq!(parser.parse(&mut src1).await.unwrap(), 0xdeadb33f);
///
/// let mut src2 = stream::from_iter("-0o755".chars())
///     .positioned::<usize>()
///     .buffered_rewind();
/// assert_eq!(parser.parse(&mut src2).await.unwrap(), -0o755);
///
/// let mut src3 = stream::from_iter("+0b00001111".chars())
///     .positioned::<usize>()
///     .buffered_rewind();
/// assert_eq!(parser.parse(&mut src3).await.unwrap(), 0b00001111);
///
/// let mut src4 = stream::from_iter("-425".chars())
///     .positioned::<usize>()
///     .buffered_rewind();
/// assert_eq!(parser.parse(&mut src4).await.unwrap(), -425);
///
/// // Trailing zeros is not allowed.
/// let mut src5 = stream::from_iter("050".chars())
///     .positioned::<usize>()
///     .buffered_rewind();
/// assert!(parser.parse(&mut src5).await.is_err());
/// # });
/// ```
#[macro_export]
macro_rules! int_parser {
    ($($prefix:literal => $radix:expr,)* _ => $rad:expr $(,)?) => {
        |neg: bool| somen::parser::choice((
            $(
                somen::parser::combinator::Prefix::new(
                    somen::parser::tag($prefix),
                    $crate::numeric::integer_trailing_zeros($radix, neg),
                ),
            )*
            $crate::numeric::integer($rad, neg),
        ))
    };
}

/// Takes a function returns a integer parser, returns a parser of signed integer.
///
/// The taken function must return negative result if the argument is `true`, and vice versa.
/// If `plus_sign` is `true`, it allows plus signs besides minus signs, has no effects.
pub fn signed<N, F, P, I, C>(parser: F, plus_sign: bool) -> impl Parser<I, Output = N>
where
    N: Zero + CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u32>,
    F: Fn(bool) -> P,
    P: Parser<I, Output = N>,
    I: Input<Ok = C> + ?Sized,
    C: Character,
{
    is(move |c: &C| c.is_minus() || (plus_sign && c.is_plus()))
        .expect(if plus_sign {
            "a plus or minus sign"
        } else {
            "a minus sign"
        })
        .opt()
        .then(move |sign: Option<C>| match sign {
            Some(c) if c.is_minus() => parser(true),
            _ => parser(false),
        })
        .expect("a signed number")
}

/// Takes a function returns a integer parser, returns a parser of unsigned, positive integer.
///
/// This function is for symmetry with the function [`signed`], so like it, the taken function
/// must return negative result if the argument is `true`, and vice versa.
#[inline]
pub fn unsigned<N, F, P, I, C>(parser: F) -> impl Parser<I, Output = N>
where
    N: Zero + CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u32>,
    F: Fn(bool) -> P,
    P: Parser<I, Output = N>,
    I: Input<Ok = C> + ?Sized,
    C: Character,
{
    parser(false)
}

/// An integer with given radix which has no trailing zeros.
pub fn integer<N, I, C>(radix: u32, neg: bool) -> impl Parser<I, Output = N>
where
    N: Zero + CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u32>,
    I: Input<Ok = C> + ?Sized,
    C: Character,
{
    integer_inner(
        (non_zero_digit(radix).once(), digit(radix).repeat(..))
            .or(is(|c: &C| c.is_zero()).expect("zero").once()),
        radix,
        neg,
    )
}

/// An integer with given radix which allows trailing zeros.
pub fn integer_trailing_zeros<N, I, C>(radix: u32, neg: bool) -> impl Parser<I, Output = N>
where
    N: Zero + CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u32>,
    I: Input<Ok = C> + ?Sized,
    C: Character,
{
    integer_inner(digit(radix).repeat(1..), radix, neg)
}

fn integer_inner<N, S, I, C>(streamed: S, radix: u32, neg: bool) -> impl Parser<I, Output = N>
where
    N: Zero + CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u32>,
    S: StreamedParser<I, Item = C>,
    I: Positioned<Ok = C> + ?Sized,
    C: Character,
{
    let integer = streamed.try_fold(value_fn(N::zero), move |acc, x| {
        N::try_from(radix)
            .ok()
            .and_then(|ten| acc.checked_mul(&ten))
            .zip(N::try_from(x.to_digit(radix)).ok().and_then(|x| {
                if neg {
                    x.checked_neg()
                } else {
                    Some(x)
                }
            }))
            .and_then(|(a, x)| a.checked_add(&x))
            .ok_or("a not too large number")
    });

    #[cfg(feature = "alloc")]
    {
        integer.expect(format!("an integer with radix {radix}"))
    }
    #[cfg(not(feature = "alloc"))]
    {
        digit.expect("an integer")
    }
}

/// Parses a digit with given radix.
pub fn digit<I, C>(radix: u32) -> impl Parser<I, Output = C>
where
    I: Positioned<Ok = C> + ?Sized,
    C: Character,
{
    let digit = is(move |c: &C| c.is_digit(radix));

    #[cfg(feature = "alloc")]
    {
        digit.expect(format!("a digit with radix {radix}"))
    }
    #[cfg(not(feature = "alloc"))]
    {
        digit.expect("a digit")
    }
}

/// Parses a non-zero digit with given radix.
pub fn non_zero_digit<I, C>(radix: u32) -> impl Parser<I, Output = C>
where
    I: Positioned<Ok = C> + ?Sized,
    C: Character,
{
    let digit = is(move |c: &C| c.is_digit(radix) && !c.is_zero());

    #[cfg(feature = "alloc")]
    {
        digit.expect(format!("a non-zero digit with radix {radix}"))
    }
    #[cfg(not(feature = "alloc"))]
    {
        digit.expect("a non-zero digit")
    }
}
