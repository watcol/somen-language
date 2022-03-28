//! Parsers for numeric literals.
use somen::prelude::*;

use crate::Character;

pub mod integer;
pub mod float;

/// Takes a function returns a integer parser, returns a parser of signed integer.
///
/// The taken function must return negative result if the argument is `true`, and vice versa.
/// If `plus_sign` is `true`, it allows plus signs besides minus signs, has no effects.
pub fn signed<'a, N, F, P, I, C>(parser: F, plus_sign: bool) -> impl Parser<I, Output = N> + 'a
where
    F: Fn(bool) -> P + 'a,
    P: Parser<I, Output = N> + 'a,
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    is(move |c: &C| c.is_minus() || (plus_sign && c.is_plus()))
        .expect(if plus_sign {
            "a plus or minus sign"
        } else {
            "a minus sign"
        })
        .opt()
        .then(move |sign: Option<C>| match sign {
            Some(c) if c.is_minus() => parser(true),
            _ => parser(false),
        })
        .expect("a signed number")
}

/// Takes a function returns a integer parser, returns a parser of unsigned, positive integer.
///
/// This function is for symmetry with the function [`signed`], so like it, the taken function
/// must return negative result if the argument is `true`, and vice versa.
#[inline]
pub fn unsigned<'a, N, F, P, I, C>(parser: F) -> impl Parser<I, Output = N> + 'a
where
    F: Fn(bool) -> P,
    P: Parser<I, Output = N> + 'a,
    I: Positioned<Ok = C> + ?Sized,
    C: Character,
{
    parser(false)
}

/// Parses a digit with given radix.
pub fn digit<'a, I, C>(radix: u32) -> impl Parser<I, Output = C> + 'a
where
    I: Positioned<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    let digit = is(move |c: &C| c.is_digit(radix));

    #[cfg(feature = "alloc")]
    {
        digit.expect(format!("a digit with radix {radix}"))
    }
    #[cfg(not(feature = "alloc"))]
    {
        digit.expect("a digit")
    }
}

/// Parses a non-zero digit with given radix.
pub fn non_zero_digit<'a, I, C>(radix: u32) -> impl Parser<I, Output = C> + 'a
where
    I: Positioned<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    let digit = is(move |c: &C| c.is_digit(radix) && !c.is_zero());

    #[cfg(feature = "alloc")]
    {
        digit.expect(format!("a non-zero digit with radix {radix}"))
    }
    #[cfg(not(feature = "alloc"))]
    {
        digit.expect("a non-zero digit")
    }
}
