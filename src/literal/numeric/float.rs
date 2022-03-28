//! Parsers for floating point decimals.
use core::ops::Neg;
use core::str::FromStr;
use somen::prelude::*;

use super::{digit, non_zero_digit};
use crate::Character;

/// A floating point number.
///
/// Note that this function doesn't support infinities or `NaN`s, you must implement it by
/// yourself.
// TODO: Implement Eisel-Lemire algolithm.
#[cfg(feature = "alloc")]
pub fn float<'a, N, I>(neg: bool) -> impl Parser<I, Output = N> + 'a
where
    N: FromStr + Neg<Output = N>,
    I: Input<Ok = char> + 'a,
{
    let integer = (non_zero_digit(10).once(), digit(10).repeat(..))
        .or(is(Character::is_zero).expect("zero").once());
    let decimal = (
        is(Character::is_point).expect("a decimal point").once(),
        digit(10).repeat(1..),
    );
    let exponent = (
        is(Character::is_exp).expect("a exponent mark").once(),
        choice((
            is(Character::is_plus).expect("a plus sign"),
            is(Character::is_minus).expect("a minus sign"),
        ))
        .once()
        .opt(),
        digit(10).repeat(1..),
    );

    (integer, decimal.opt(), exponent.opt())
        .collect::<alloc::string::String>()
        .try_map(|s| N::from_str(s.as_str()).or(Err("a valid floating point number")))
        .map(move |i| if neg { -i } else { i })
}
