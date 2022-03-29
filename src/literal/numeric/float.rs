//! Parsers for floating point decimals.
use core::ops::{Mul, Neg};
use num_traits::Pow;
use somen::prelude::*;

use super::integer::integer_inner;
use super::{digits, digits_trailing_zeros, signed};
use crate::Character;

/// A floating point number.
///
/// Note that this function doesn't support infinities or `NaN`s, you must implement it by
/// yourself.
// TODO: Implement Eisel-Lemire algolithm.
pub fn float<'a, N, I, C>(neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: Mul<N, Output = N> + Neg<Output = N> + Pow<i32, Output = N> + TryFrom<u64> + 'a,
    I: Input<Ok = C> + 'a,
    C: Character + 'a,
{
    float_inner().try_map(move |(mantissa, _, exponent, _)| {
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

fn float_inner<'a, I, C>() -> impl Parser<I, Output = (u64, bool, i32, bool)> + 'a
where
    I: Input<Ok = C> + 'a,
    C: Character + 'a,
{
    integer_inner::<u64, _, _, _>(digits(10), 0, 10, false)
        .then(|(int, _, overflowed)| {
            if overflowed {
                value((int, 0, true)).left()
            } else {
                is(Character::is_point)
                    .expect("a decimal point")
                    .prefix(integer_inner(digits_trailing_zeros(10), int, 10, false))
                    .or(value((int, 0, false)))
                    .right()
            }
        })
        .and(
            is(Character::is_exp)
                .expect("a exponent mark")
                .prefix(signed(
                    |neg| integer_inner(digits_trailing_zeros(10), 0i32, 10, neg),
                    true,
                ))
                .or(value((0, 0, false))),
        )
        .map(
            |((mantissa, count, man_overflowed), (exp, _, mut exp_overflowed))| {
                let exp = if exp_overflowed {
                    if exp < 0 {
                        i32::MIN
                    } else {
                        i32::MAX
                    }
                } else {
                    let res = exp.saturating_sub(count as i32);
                    if res == i32::MIN {
                        exp_overflowed = true;
                    }
                    res
                };
                (
                    if man_overflowed { u64::MAX } else { mantissa },
                    man_overflowed,
                    exp,
                    exp_overflowed,
                )
            },
        )
}
