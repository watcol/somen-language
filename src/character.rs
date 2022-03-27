/// A trait for characters.
pub trait Character: Clone {
    /// Checks if the character is zero (`0`) or not.
    fn is_zero(&self) -> bool;

    /// Checks if the character is plus sign (`+`) or not.
    fn is_plus(&self) -> bool;

    /// Checks if the character is minus sign (`-`) or not.
    fn is_minus(&self) -> bool;

    /// Checks if the character is a digit with the radix, or not.
    ///
    /// # Panics
    /// if `radix` is not in range of 2 to 36.
    fn is_digit(&self, radix: u32) -> bool;

    /// Converts the character satisfies [`is_digit`] into an integer value.
    ///
    /// # Panics
    /// if `radix` is not in range of 2 to 36, or the character not satisfies [`is_digit`].
    ///
    /// [`is_digit`]: Self::is_digit
    fn to_digit(&self, radix: u32) -> u32;
}

impl Character for char {
    #[inline]
    fn is_zero(&self) -> bool {
        *self == '0'
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
    fn is_digit(&self, radix: u32) -> bool {
        Self::is_digit(*self, radix)
    }

    #[inline]
    fn to_digit(&self, radix: u32) -> u32 {
        Self::to_digit(*self, radix).unwrap()
    }
}
