/// A trait for characters.
pub trait Character: Clone {
    /// Checks if the character is zero (`0`) or not.
    fn is_zero(&self) -> bool;

    /// Checks if the character is a decimal point (`.`) or not.
    fn is_point(&self) -> bool;

    /// Checks if the character is a exponent mark (`e`/`E`) or not.
    fn is_exp(&self) -> bool;

    /// Checks if the character is plus sign (`+`) or not.
    fn is_plus(&self) -> bool;

    /// Checks if the character is minus sign (`-`) or not.
    fn is_minus(&self) -> bool;

    /// Checks if the character is a digit with the radix, or not.
    ///
    /// # Panics
    /// if `radix` is not in range of 2 to 36.
    fn is_digit(&self, radix: u8) -> bool;

    /// Converts the character satisfies [`is_digit`] into an integer value.
    ///
    /// # Panics
    /// if `radix` is not in range of 2 to 36, or the character not satisfies [`is_digit`].
    ///
    /// [`is_digit`]: Self::is_digit
    fn to_digit_unchecked(&self, radix: u8) -> u8;

    /// Converts the character into an integer value or return [`None`] if this is not a digit.
    ///
    /// # Panics
    /// if `radix` is not in range of 2 to 36.
    #[inline]
    fn to_digit(&self, radix: u8) -> Option<u8> {
        if self.is_digit(radix) {
            Some(self.to_digit_unchecked(radix))
        } else {
            None
        }
    }
}

impl Character for char {
    #[inline]
    fn is_zero(&self) -> bool {
        *self == '0'
    }

    #[inline]
    fn is_point(&self) -> bool {
        *self == '.'
    }

    #[inline]
    fn is_exp(&self) -> bool {
        *self == 'e' || *self == 'E'
    }

    #[inline]
    fn is_plus(&self) -> bool {
        *self == '+'
    }

    #[inline]
    fn is_minus(&self) -> bool {
        *self == '-'
    }

    #[inline]
    fn is_digit(&self, radix: u8) -> bool {
        Self::is_digit(*self, radix as u32)
    }

    #[inline]
    fn to_digit_unchecked(&self, radix: u8) -> u8 {
        Self::to_digit(*self, radix as u32).unwrap() as u8
    }

    #[inline]
    fn to_digit(&self, radix: u8) -> Option<u8> {
        Self::to_digit(*self, radix as u32).map(|d| d as u8)
    }
}
