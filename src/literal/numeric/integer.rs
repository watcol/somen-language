//! Parsers for integers.
use num_traits::{CheckedAdd, CheckedMul, CheckedNeg, Zero};
use somen::prelude::*;

use super::{digit, non_zero_digit};
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
    integer_inner(
        (non_zero_digit(radix).once(), digit(radix).repeat(..))
            .or(is(|c: &C| c.is_zero()).expect("zero").once()),
        N::zero(),
        radix,
        neg,
    )
    .try_map(|x| x.map(|(n, _)| n).ok_or("a not too large number."))
}

/// An integer with given radix which allows trailing zeros.
pub fn integer_trailing_zeros<'a, N, I, C>(radix: u8, neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: Zero + CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u8> + Clone + 'a,
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    integer_inner(digit(radix).repeat(1..), N::zero(), radix, neg)
        .try_map(|x| x.map(|(n, _)| n).ok_or("a not too large number."))
}

pub(super) fn integer_inner<'a, N, S, I, C>(
    streamed: S,
    default: N,
    radix: u8,
    neg: bool,
) -> impl Parser<I, Output = Option<(N, u8)>> + 'a
where
    N: CheckedMul + CheckedAdd + CheckedNeg + TryFrom<u8> + Clone + 'a,
    S: StreamedParser<I, Item = C> + 'a,
    I: Positioned<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    let integer = streamed.fold(value(Some((default, 0))), move |acc, x| {
        let mut x = N::try_from(x.to_digit_unchecked(radix)).ok()?;
        if neg {
            x = x.checked_neg()?;
        }
        let (n, count) = acc?;
        Some((
            n.checked_mul(&N::try_from(radix).ok()?)?.checked_add(&x)?,
            count + 1,
        ))
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
