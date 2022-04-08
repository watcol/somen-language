//! Parsers for floating point decimals.
use somen::prelude::*;

use super::integer::fold_digits;
use super::{digits, digits_trailing_zeros, signed};
use crate::character::{character, Character};

use compute_float::compute_float;

/// A floating point number.
///
/// This parser requires an integer part, but a decimal part and an exponent are optional.
/// If you want to apply different rules, you can implement it by yourself using [`compute_float`]
/// helper function.
///
/// Also note that this function doesn't support infinities and NaNs.
#[cfg(any(feature = "std", feature = "libm"))]
#[inline]
pub fn float<'a, N, I, C>(neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: compute_float::Float + num_traits::Float + 'a,
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    float_inner().map(move |(man, exp10)| {
        compute_float(neg, man, exp10).unwrap_or_else(|| {
            let res = N::from(man).unwrap() * N::from(10u8).unwrap().powi(exp10);
            if neg {
                -res
            } else {
                res
            }
        })
    })
}

#[inline]
#[cfg(not(any(feature = "std", feature = "libm")))]
pub fn float<'a, N, I, C>(neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: compute_float::Float + 'a,
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    float_inner().try_map(move |(man, exp10)| {
        compute_float(neg, man, exp10).ok_or("a valid floating point number")
    })
}

fn float_inner<'a, I, C>() -> impl Parser<I, Output = (u64, i32)> + 'a
where
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    fold_digits::<u64, _, _, _>(digits(10), 0, 10, false)
        .then(|(int, _, overflowed)| {
            if overflowed {
                value((int, 0, true)).left()
            } else {
                character(b'.')
                    .prefix(fold_digits(digits_trailing_zeros(10), int, 10, false))
                    .or(value((int, 0, false)))
                    .right()
            }
        })
        .and(
            character(b'e')
                .or(character(b'E'))
                .prefix(signed(
                    |neg| fold_digits(digits_trailing_zeros(10), 0i32, 10, neg),
                    true,
                ))
                .or(value((0, 0, false))),
        )
        .map(
            |((mantissa, count, man_overflowed), (exp, _, exp_overflowed))| {
                (
                    if man_overflowed { u64::MAX } else { mantissa },
                    if exp_overflowed {
                        if exp < 0 {
                            i32::MIN
                        } else {
                            i32::MAX
                        }
                    } else {
                        exp.saturating_sub(count as i32)
                    },
                )
            },
        )
}
