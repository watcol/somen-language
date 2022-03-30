//! Utilities of the somen parser combinator for languages.

#![cfg_attr(not(feature = "std"), no_std)]
#![doc(test(attr(warn(warnings))))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod character;
pub mod token;
