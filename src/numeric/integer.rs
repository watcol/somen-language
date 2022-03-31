//! Parsers for integers.
use num_traits::{CheckedAdd, CheckedMul, CheckedNeg, Zero};
use somen::prelude::*;

use super::{digits, digits_fixed, digits_trailing_zeros};
use crate::character::Character;

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
