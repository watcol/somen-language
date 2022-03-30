//! Parsers for integers.
use num_traits::{CheckedAdd, CheckedMul, CheckedNeg, Zero};
use somen::prelude::*;

use super::{digits, digits_trailing_zeros};
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
/// use somen_language::literal::numeric::signed;
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
                    $crate::literal::numeric::integer::integer_trailing_zeros($radix, neg),
                ),
            )*
            $crate::literal::numeric::integer::integer($rad, neg),
        ))
    };
}

/// An integer with given radix which has no trailing zeros.
pub fn integer<'a, N, I, C>(radix: u8, neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: Zero + CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u8> + Clone + 'a,
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    integer_inner(digits(radix), N::zero(), radix, neg).try_map(|(acc, _, overflowed)| {
        if overflowed {
            Err("a not too large number.")
        } else {
            Ok(acc)
        }
    })
}

/// An integer with given radix which allows trailing zeros.
pub fn integer_trailing_zeros<'a, N, I, C>(radix: u8, neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: Zero + CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u8> + Clone + 'a,
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    integer_inner(digits_trailing_zeros(radix), N::zero(), radix, neg).try_map(
        |(acc, _, overflowed)| {
            if overflowed {
                Err("a not too large number.")
            } else {
                Ok(acc)
            }
        },
    )
}

pub(super) fn integer_inner<'a, N, S, I, C>(
    streamed: S,
    default: N,
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
    let integer = streamed.fold(
        value((default, 0, n_radix.is_none())),
        move |(acc, count, overflowed), x| {
            if overflowed {
                return (acc, count, true);
            }
            let res = acc
                .checked_mul(n_radix.as_ref().unwrap())
                .zip(N::try_from(x.to_digit_unchecked(radix)).ok().and_then(|x| {
                    if neg {
                        x.checked_neg()
                    } else {
                        Some(x)
                    }
                }))
                .and_then(|(acc, x)| acc.checked_add(&x));
            match res {
                Some(x) => (x, count + 1, false),
                None => (acc, count, true),
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
