//! Parsers for floating point decimals.
use core::ops::{Add, Mul, Neg};
use num_traits::{Pow, Zero};
use somen::prelude::*;

use super::integer::{integer, integer_trailing_zeros};
use super::{digit, signed};
use crate::Character;

/// A floating point number.
///
/// Note that this function doesn't support infinities or `NaN`s, you must implement it by
/// yourself.
// TODO: Implement Eisel-Lemire algolithm.
#[cfg(feature = "alloc")]
pub fn float<'a, N, I, C>(neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: Zero
        + Add<N, Output = N>
        + Pow<i32, Output = N>
        + Mul<N, Output = N>
        + Neg<Output = N>
        + TryFrom<u64>
        + Clone
        + 'a,
    I: Input<Ok = C> + 'a,
    C: Character + 'a,
{
    let integer = integer(10, false);
    let fraction = is(Character::is_point)
        .expect("a decimal point")
        .prefix(digit(10).repeat(1..))
        .try_fold::<_, _, &str>(value((N::zero(), 0)), |(acc, offset), digit: C| {
            Ok((
                acc + N::try_from(10).or(Err("a valid float"))?.pow(offset - 1)
                    * N::try_from(digit.to_digit(10) as u64).or(Err("a valid float"))?,
                offset - 1,
            ))
        })
        .map(|(frac, _)| frac);
    let exponent = is(Character::is_exp)
        .expect("a exponent mark")
        .prefix(signed(|neg| integer_trailing_zeros(10, neg), true));

    (integer, fraction.opt(), exponent.opt()).try_map::<_, _, &str>(move |(int, frac, exp)| {
        let mut int = N::try_from(int).or(Err("a not too large number."))?;
        if let Some(f) = frac {
            int = int + f;
        }
        if let Some(e) = exp {
            int = int * N::try_from(10).or(Err("a valid float"))?.pow(e);
        }
        if neg {
            int = -int;
        }
        Ok(int)
    })
}
