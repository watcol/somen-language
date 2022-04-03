//! Parsers for numeric literals.
use somen::prelude::*;

use crate::character::{character, Character};

pub mod float;
pub mod integer;

/// Takes a function returns a integer parser, returns a parser of signed integer.
///
/// The taken function must return negative result if the argument is `true`, and vice versa.
/// If `plus_sign` is `true`, it allows plus signs besides minus signs, has no effects.
pub fn signed<'a, N, F, P, I, C>(mut parser: F, plus_sign: bool) -> impl Parser<I, Output = N> + 'a
where
    F: FnMut(bool) -> P + 'a,
    P: Parser<I, Output = N> + 'a,
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    sign(plus_sign)
        .opt()
        .then(move |sign| parser(sign.unwrap_or_default()))
        .expect("a signed number")
}

/// Takes a function returns a integer parser, returns a parser of unsigned, positive integer.
///
/// This function is for symmetry with the function [`signed`], so like it, the taken function
/// must return negative result if the argument is `true`, and vice versa.
#[inline]
pub fn unsigned<'a, N, F, P, I, C>(parser: F) -> impl Parser<I, Output = N> + 'a
where
    F: FnOnce(bool) -> P,
    P: Parser<I, Output = N> + 'a,
    I: Positioned<Ok = C> + ?Sized,
    C: Character,
{
    parser(false)
}

/// Parses a digit with given radix.
pub fn digit<'a, I, C>(radix: u8) -> impl Parser<I, Output = C> + 'a
where
    I: Positioned<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    let digit = is(move |c: &C| c.is_digit(radix));

    #[cfg(feature = "alloc")]
    {
        digit.expect(alloc::format!("a digit with radix {radix}"))
    }
    #[cfg(not(feature = "alloc"))]
    {
        digit.expect("a digit")
    }
}

/// Parses a non-zero digit with given radix.
pub fn non_zero_digit<'a, I, C>(radix: u8) -> impl Parser<I, Output = C> + 'a
where
    I: Positioned<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    let digit = is(move |c: &C| c.is_digit(radix) && !c.eq_byte(b'0'));

    #[cfg(feature = "alloc")]
    {
        digit.expect(alloc::format!("a non-zero digit with radix {radix}"))
    }
    #[cfg(not(feature = "alloc"))]
    {
        digit.expect("a non-zero digit")
    }
}

/// Parses digits with given radix.
#[inline]
pub fn digits<'a, I, C>(radix: u8) -> impl IterableParser<I, Item = C> + 'a
where
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    (non_zero_digit(radix).once(), digit(radix).repeat(..)).or(character(b'0').once())
}

/// Parses digits with given radix (trailing zeros are allowed).
#[inline]
pub fn digits_trailing_zeros<'a, I, C>(radix: u8) -> impl IterableParser<I, Item = C> + 'a
where
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    digit(radix).repeat(1..)
}

/// Parses fixed-length digits with given radix.
#[inline]
pub fn digits_fixed<'a, I, C>(length: usize, radix: u8) -> impl IterableParser<I, Item = C> + 'a
where
    I: Positioned<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    digit(radix).times(length)
}

/// Parses a plus or minus sign and returns `true` if it is a minus sign.
///
/// By the default, plus signs are not allowed and it will be allowed if the argument `plus_sign` is `true`.
pub fn sign<'a, I, C>(plus_sign: bool) -> impl Parser<I, Output = bool> + 'a
where
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    let minus = character(b'-').map(|_| true);
    if plus_sign {
        minus.or(character(b'+').map(|_| false)).left()
    } else {
        minus.right()
    }
}
