/// Generates a function returns a integer parser.
///
/// This macro takes comma-separated list of patterns `prefix => radix` like `tag("0x") => 16`, and
/// `_ => radix` in the last are interpreted as a fallback without prefix.
#[macro_export]
macro_rules! int_parser {
    ($($prefix:expr => $radix:expr,)* _ => $rad:expr $(,)?) => {
        |neg: bool| somen::parser::choice((
            $(
                somen::parser::combinator::Prefix::new(
                    $prefix,
                    $crate::token::numeric::integer::integer_trailing_zeros($radix, neg),
                ),
            )*
            $crate::token::numeric::integer::integer($rad, neg),
        ))
    };
}

/// Generate enums represents symbols.
///
/// This macro is very like [`token`], but fields of variants are disallowed.
#[macro_export]
macro_rules! symbol {
    ($(#[$attrs:meta])* $vis:vis enum $name:ident $(: $src:ty)? {
        $($var:ident = $token:expr),+ $(,)?
    }) => {
        $(#[$attrs])*
        $vis enum $name {
            $($var,)+
        }

        impl $name {
            #[allow(dead_code)]
            pub fn parser<'a, I>() -> impl somen::parser::Parser<I, Output = $name> + 'a
            where
                I: Input$(<Ok=$src>)? + ?Sized + 'a,
            {
                somen::parser::choice((
                    $(
                        somen::parser::wrapper::Map::new(
                            $token,
                            |_| $name::$var
                        ),
                    )+
                ))
            }
        }
    }
}

/// Generate enums represents tokens or syntax trees.
///
/// In this macro, each variants must be formed `Variant()`, `Variant {}` (both ignores parser outputs),
/// `Variant(Type)` (parser outputs should be typed as `Type`) or `Variant { foo: Foo, bar: Bar }`
/// (parser outputs should be typed as `(Foo, Bar, ...)`). For parser of symbols which requires no
/// outputs, use the macro [`symbol`] which has no fields for variants.
#[macro_export]
macro_rules! token {
    ($(#[$attrs:meta])* $vis:vis enum $name:ident $(: $src:ty)? {
        $($var:ident $field:tt = $token:expr),+ $(,)?
    }) => {
        $(#[$attrs])*
        $vis enum $name {
            $($var $field,)+
        }

        impl $name {
            #[allow(dead_code)]
            pub fn parser<'a, I>() -> impl somen::parser::Parser<I, Output = $name> + 'a
            where
                I: Input$(<Ok=$src>)? + ?Sized + 'a,
            {
                somen::parser::choice((
                    $(
                        somen::parser::wrapper::Map::new(
                            $token,
                            $crate::__token_inner!($name $var; $field)
                        ),
                    )+
                ))
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __token_inner {
    ($name:ident $var:ident;()) => {
        |_| $name::$var()
    };
    ($name:ident $var:ident; ($inner:ty)) => {
        |inner| $name::$var(inner)
    };
    ($name:ident $var:ident; { $($field:ident : $ty:ty),* $(,)? }) => {
        |($($field,)*)| $name::$var { $($field,)* }
    }
}
