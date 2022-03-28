//! Parsers for floating point decimals.
use core::ops::{Mul, Neg};
use num_traits::Pow;
use somen::prelude::*;

use super::integer::{integer_inner, integer_trailing_zeros};
use super::{digit, non_zero_digit, signed};
use crate::Character;

/// A floating point number.
///
/// Note that this function doesn't support infinities or `NaN`s, you must implement it by
/// yourself.
// TODO: Implement Eisel-Lemire algolithm.
pub fn float<'a, N, I, C>(neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: Mul<N, Output = N> + Neg<Output = N> + Pow<i64, Output = N> + TryFrom<u64> + 'a,
    I: Input<Ok = C> + 'a,
    C: Character + 'a,
{
    float_parser().try_map(move |(mantissa, exponent, _)| {
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

pub fn float_parser<'a, I, C>() -> impl Parser<I, Output = (u64, i64, bool)> + 'a
where
    I: Input<Ok = C> + 'a,
    C: Character + 'a,
{
    integer_inner::<u64, _, _, _>(
        (non_zero_digit(10).once(), digit(10).repeat(..))
            .or(is(Character::is_zero).expect("zero").once()),
        0,
        10,
        false,
    )
    .then(|(int, overflowed, _)| {
        if overflowed {
            value((int, true, 0)).left()
        } else {
            is(Character::is_point)
                .expect("a decimal point")
                .prefix(integer_inner(digit(10).repeat(1..), int, 10, false))
                .or(value((int, false, 0)))
                .right()
        }
    })
    .and(
        is(Character::is_exp)
            .expect("a exponent mark")
            .prefix(signed(|neg| integer_trailing_zeros(10, neg), true))
            .or(value(0)),
    )
    .map(|((mantissa, overflowed, count), exp2)| (mantissa, exp2 - (count as i64), overflowed))
}
