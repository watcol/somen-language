//! Abstractions for characters. (`char`, `u8`, ...)
use alloc::string::ToString;
use somen::error::{ExpectKind, Expects};
use somen::prelude::*;

/// A parser for characters.
///
/// # Panics
/// if `byte` is not an ascii character.
#[inline]
pub fn character<'a, I, C>(byte: u8) -> impl Parser<I, Output = C> + 'a
where
    I: Positioned<Ok = C> + ?Sized + 'a,
    C: Character + 'a,
{
    assert!(byte.is_ascii());
    is(move |c: &C| c.eq_byte(byte)).expect(unsafe { C::byte_to_expect_unchecked(byte) })
}

/// A trait for characters.
pub trait Character: Clone {
    /// Checks if the character equals to an ascii `byte` (in range of `0x00..=0x7F`).
    fn eq_byte(&self, byte: u8) -> bool;

    /// An unsafe version of [`byte_to_expect`]
    ///
    /// # Safety
    /// Calling this method on `!byte.is_ascii()` is undefined behavior.
    ///
    /// [`byte_to_expect`]: Self::byte_to_expect
    #[inline]
    unsafe fn byte_to_expect_unchecked(byte: u8) -> Expects<Self> {
        assert!(byte.is_ascii());
        #[cfg(feature = "alloc")]
        {
            Expects::from(ExpectKind::Owned(
                char::from_u32_unchecked(byte as u32).to_string(),
            ))
        }
        #[cfg(not(feature = "alloc"))]
        {
            Expects::from(ExpectKind::Static("a byte"))
        }
    }

    /// Converts a byte to an [`Expects`], the error specifier of the somen parser combinator.
    ///
    /// This function returns [`None`] if `byte` is not an ascii character.
    fn byte_to_expect(byte: u8) -> Option<Expects<Self>> {
        if byte.is_ascii() {
            Some(unsafe { Self::byte_to_expect_unchecked(byte) })
        } else {
            None
        }
    }

    /// Checks if the character is a digit with the radix, or not.
    ///
    /// # Panics
    /// if `radix` is not in range of 2 to 36.
    fn is_digit(&self, radix: u8) -> bool;

    /// An unsafe version of [`to_digit`].
    ///
    /// Either this or [`to_digit`] must be implemented.
    ///
    /// # Safety
    /// Calling this method on `!self.is_digit(radix)` is undefined behavior.
    ///
    /// [`to_digit`]: Self::to_digit
    #[inline]
    unsafe fn to_digit_unchecked(&self, radix: u8) -> u8 {
        self.to_digit(radix).unwrap_unchecked()
    }

    /// Converts the character into an integer value or return [`None`] if this is not a digit.
    ///
    /// Either this or [`to_digit_unchecked`] must be implemented.
    ///
    /// # Panics
    /// if `radix` is not in range of 2 to 36.
    ///
    /// [`to_digit_unchecked`]: Self::to_digit_unchecked
    #[inline]
    fn to_digit(&self, radix: u8) -> Option<u8> {
        if self.is_digit(radix) {
            Some(unsafe { self.to_digit_unchecked(radix) })
        } else {
            None
        }
    }
}

impl Character for char {
    #[inline]
    fn eq_byte(&self, byte: u8) -> bool {
        *self as u32 == byte as u32
    }

    #[inline]
    unsafe fn byte_to_expect_unchecked(byte: u8) -> Expects<Self> {
        Expects::from(ExpectKind::Token(char::from_u32_unchecked(byte as u32)))
    }

    #[inline]
    fn is_digit(&self, radix: u8) -> bool {
        Self::is_digit(*self, radix as u32)
    }

    #[inline]
    fn to_digit(&self, radix: u8) -> Option<u8> {
        Self::to_digit(*self, radix as u32).map(|d| d as u8)
    }
}
