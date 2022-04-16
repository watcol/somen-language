//! Utilities of the somen parser combinator for languages.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(doc_cfg))]
#![doc(test(attr(warn(warnings))))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod character;
pub mod identifier;
mod macros;
pub mod numeric;
