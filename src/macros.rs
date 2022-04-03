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
                    $crate::numeric::integer::integer_trailing_zeros($radix, neg),
                ),
            )*
            $crate::numeric::integer::integer($rad, neg),
        ))
    };
}

/// Generate enums represents tokens or syntax trees.
///
/// In this macro, each variants must be formed `Self::Variant` or `Self::Variant(Type)` and if a
/// parameter exists, the parser output type should be `Type`.
#[macro_export]
macro_rules! token {
    ($(#[$attrs:meta])* $vis:vis enum $name:ident : $src:ty {
        $(
            $(#[token($($key:ident = $value:ident),* $(,)?)])?
            $var:ident $(($field:ty))? = $token:expr
        ),+ $(,)?
    }) => {
        $(#[$attrs])*
        $vis enum $name {
            $($var $(($field))?,)+
        }

        impl $name {
            #[allow(dead_code)]
            pub fn parser<'a, I>() -> impl somen::parser::Parser<I, Output = $name> + 'a
            where
                I: Input<Ok=$src> + ?Sized + 'a,
            {
                somen::parser::choice((
                    $(
                        somen::parser::wrapper::Map::new(
                            $token,
                            $crate::__token_inner!{@closure [$name] [$var]; $($field)? },
                        ),
                    )+
                ))
            }

            $(
                $crate::__token_inner! {
                    @method [$name] [$var] [$($field)?] [$src] [$token];
                    $($([$key = $value])*)?
                }
             )+
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __token_inner {
    (@method [$name:ident] [$var:ident] [$($field:ty)?] [$src:ty] [$token:expr];) => {};
    (@method [$name:ident] [$var:ident] [$($field:ty)?] [$src:ty] [$token:expr];
        [match = $fname:ident]$([$k:ident = $v:ident])*) => {
        #[allow(dead_code)]
        #[inline]
        pub fn $fname<'a, I>() -> impl somen::parser::Parser<I, Output = ($($field)?)> + 'a
        where
            I: Positioned<Ok=$name> + ?Sized + 'a,
        {
            somen::parser::wrapper::Expect::new(
                somen::parser::is_some(|c|
                    $crate::__token_inner! { @match c [$name] [$var]; $($field)? }
                ),
                stringify!($fname).into(),
            )
        }

        $crate::__token_inner! { @method [$name] [$var] [$($field)?] [$src] [$token]; $([$k = $v])* }
    };
    (@method [$name:ident] [$var:ident] [$field:ty] [$src:ty] [$token:expr];
        [match_arg = $fname:ident]$([$k:ident = $v:ident])*) => {
        #[allow(dead_code)]
        #[inline]
        pub fn $fname<'a, I, T>(inner: T) -> impl somen::parser::Parser<I, Output = $field> + 'a
        where
            I: Positioned<Ok=$name> + ?Sized + 'a,
            T: PartialEq<$field> + 'a,
        {
            somen::parser::wrapper::Expect::new(
                somen::parser::is_some(move |c| match c {
                    $name::$var(val) if inner == val => Some(val),
                    _ => None,
                }),
                stringify!($fname).into(),
            )
        }

        $crate::__token_inner! { @method [$name] [$var] [$field] [$src] [$token]; $([$k = $v])* }
    };
    (@method [$name:ident] [$var:ident] [] [$src:ty] [$token:expr];
        [match_arg = $fname:ident]$([$k:ident = $v:ident])*) => {
        compile_error!("`match_arg` is not supported for variants without fields.");

        $crate::__token_inner! { @method [$name] [$var] [] [$src] [$token]; $([$k = $v])* }
    };
    (@method [$name:ident] [$var:ident] [$($field:ty)?] [$src:ty] [$token:expr];
        [single = $fname:ident]$([$k:ident = $v:ident])*) => {
        #[allow(dead_code)]
        #[inline]
        pub fn $fname<'a, I>() -> impl somen::parser::Parser<I, Output = $name> + 'a
        where
            I: Input<Ok=$src> + ?Sized + 'a,
        {
            somen::parser::wrapper::Map::new(
                $token,
                $crate::__token_inner! { @closure [$name] [$var]; $($field)? },
            )
        }

        $crate::__token_inner! { @method [$name] [$var] [$($field)?] [$src] [$token]; $([$k = $v])* }
    };
    (@match $c:ident [$name:ident] [$var:ident];) => {
        match $c {
            $name::$var => Some(()),
            _ => None,
        }
    };
    (@match $c:ident [$name:ident] [$var:ident]; $field:ty) => {
        match $c {
            $name::$var(inner) => Some(inner),
            _ => None,
        }
    };
    (@closure [$name:ident] [$var:ident];) => {
        |_| $name::$var
    };
    (@closure [$name:ident] [$var:ident]; $field:ty) => {
        $name::$var
    };
}
