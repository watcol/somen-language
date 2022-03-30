//! Parsers for floating point decimals.
use core::ops::{Mul, Neg};
use num_traits::Pow;
use somen::prelude::*;

use super::integer::fold_digits;
use super::{digits, digits_trailing_zeros, signed};
use crate::character::{character, Character};

/// A floating point number.
///
/// This parser requires an integer part, but a decimal part and an exponent are optional.
/// If you want to apply different rules, you can implement it by yourself using [`compute_float`]
/// helper function.
///
/// Also note that this function doesn't support infinities and NaNs.
#[inline]
pub fn float<'a, N, I, C>(neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: Mul<N, Output = N> + Neg<Output = N> + Pow<i32, Output = N> + TryFrom<u64> + 'a,
    I: Input<Ok = C> + 'a,
    C: Character + 'a,
{
    compute_float(float_inner(), neg)
}

/// Takes an parser for floats which returns `(mantissa, exponent)`, compute a float with it.
// TODO: Implement Eisel-Lemire algolithm.
pub fn compute_float<'a, N, P, I, C>(parser: P, neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: Mul<N, Output = N> + Neg<Output = N> + Pow<i32, Output = N> + TryFrom<u64> + 'a,
    P: Parser<I, Output = (u64, i32)> + 'a,
    I: Input<Ok = C> + 'a,
    C: Character + 'a,
{
    parser.try_map(move |(mantissa, exponent)| {
        N::try_from(mantissa)
            .ok()
            .zip(N::try_from(10).ok())
            .map(|(m, ten)| {
                let n = m * ten.pow(exponent);
                if neg {
                    -n
                } else {
                    n
                }
            })
            .ok_or("a valid float")
    })
}

fn float_inner<'a, I, C>() -> impl Parser<I, Output = (u64, i32)> + 'a
where
    I: Input<Ok = C> + 'a,
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
