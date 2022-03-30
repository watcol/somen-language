//! Parsers for identifiers.
use somen::prelude::*;

use crate::character::Character;

/// Parses identifiers.
#[inline]
pub fn identifier<'a, P, Q, E, I, C>(start: P, rest: Q) -> impl Parser<I, Output = E> + 'a
where
    P: Parser<I, Output = C> + 'a,
    Q: Parser<I, Output = C> + 'a,
    E: Extend<C> + Default + 'a,
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    (start.once(), rest.repeat(..)).collect()
}

/// Parses standard identifiers starts with a letter and rest are letters, digits
/// or underscores.
#[inline]
pub fn standard_identifier<'a, E, I, C>() -> impl Parser<I, Output = E> + 'a
where
    E: Extend<C> + Default + 'a,
    I: Input<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    identifier(
        is(Character::is_letter),
        is(|c: &C| c.is_letter() || c.is_digit(10) || c.eq_byte(b'_')),
    )
}
