//! Parsers for integers.
use num_traits::{CheckedAdd, CheckedMul, CheckedNeg, Zero};
use somen::prelude::*;

use super::{digits, digits_fixed, digits_trailing_zeros};
use crate::character::Character;

/// Generates a function returns a integer parser.
///
/// This macro takes comma-separated list of patterns `prefix => radix` like `tag("0x") => 16`, and
/// `_ => radix` in the last are interpreted as a fallback without prefix.
///
/// # Examples
/// ```
/// # futures::executor::block_on(async {
/// use somen::prelude::*;
/// use somen_language::token::numeric::signed;
/// use somen_language::int_parser;
///
/// let mut parser = signed::<i64, _, _, _, _>(
///     int_parser!{
///         tag("0x") => 16,
///         tag("0o") => 8,
///         tag("0b") => 2,
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
    ($($prefix:expr => $radix:expr,)* _ => $rad:expr $(,)?) => {
        |neg: bool| somen::parser::choice((
            $(
                somen::parser::combinator::Prefix::new(
                    $prefix,
                    $crate::token::numeric::integer::integer_trailing_zeros($radix, neg),
                ),
            )*
            $crate::token::numeric::integer::integer($rad, neg),
        ))
    };
}

/// An integer with given radix which has no trailing zeros.
#[inline]
pub fn integer<'a, N, I, C>(radix: u8, neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: Zero + CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u8> + Clone + 'a,
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    let integer =
        fold_digits(digits(radix), N::zero(), radix, neg).try_map(|(acc, _, overflowed)| {
            if overflowed {
                Err("a not too large number.")
            } else {
                Ok(acc)
            }
        });

    #[cfg(feature = "alloc")]
    {
        integer.expect(alloc::format!("an integer with radix {radix}"))
    }
    #[cfg(not(feature = "alloc"))]
    {
        integer.expect("an integer")
    }
}

/// An integer with given radix which allows trailing zeros.
#[inline]
pub fn integer_trailing_zeros<'a, N, I, C>(radix: u8, neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: Zero + CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u8> + Clone + 'a,
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    let integer = fold_digits(digits_trailing_zeros(radix), N::zero(), radix, neg).try_map(
        |(acc, _, overflowed)| {
            if overflowed {
                Err("a not too large number.")
            } else {
                Ok(acc)
            }
        },
    );

    #[cfg(feature = "alloc")]
    {
        integer.expect(alloc::format!("an integer with radix {radix}"))
    }
    #[cfg(not(feature = "alloc"))]
    {
        integer.expect("an integer")
    }
}

/// A fixed-length integer with given radix.
#[inline]
pub fn integer_fixed<'a, N, I, C>(
    length: usize,
    radix: u8,
    neg: bool,
) -> impl Parser<I, Output = N> + 'a
where
    N: Zero + CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u8> + Clone + 'a,
    I: Positioned<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    fold_digits(digits_fixed(length, radix), N::zero(), radix, neg).try_map(
        |(acc, _, overflowed)| {
            if overflowed {
                Err("a not too large number.")
            } else {
                Ok(acc)
            }
        },
    )
}

/// Takes a streamed parser of digits, folds it to an `acc` as following digits.
///
/// The output value consists of a folded result of `acc`, a number of folded digits and the last
/// item will be `true` is the number has overflowed.
pub fn fold_digits<'a, N, S, I, C>(
    streamed: S,
    acc: N,
    radix: u8,
    neg: bool,
) -> impl Parser<I, Output = (N, usize, bool)> + 'a
where
    N: CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u8> + Clone + 'a,
    S: StreamedParser<I, Item = C> + 'a,
    I: Positioned<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    let n_radix = N::try_from(radix).ok();
    streamed.fold(
        value((acc, 0, n_radix.is_none())),
        move |(acc, count, overflowed), x| {
            if overflowed {
                return (acc, count, true);
            }
            let res = acc
                .checked_mul(n_radix.as_ref().unwrap())
                .zip(
                    x.to_digit(radix)
                        .and_then(|d| N::try_from(d).ok())
                        .and_then(|x| if neg { x.checked_neg() } else { Some(x) }),
                )
                .and_then(|(acc, x)| acc.checked_add(&x));
            match res {
                Some(x) => (x, count + 1, false),
                None => (acc, count, true),
            }
        },
    )
}
