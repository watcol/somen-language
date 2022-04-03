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
            $(#[atomic = $atomic:ident])?
            $(#[parser = $parser:ident])?
            $var:ident $(($field:ty))? = $token:expr
        ),+ $(,)?
    }) => {
        $(#[$attrs])*
        $vis enum $name {
            $($var $(($field))?,)+
        }

        impl $name {
            $(
                $crate::__token_inner! { @atomic [$name] [$var] [$($field)?]; $($atomic)? }
                $crate::__token_inner! { @parser [$name] [$var] [$($field)?] [$src] [$token]; $($parser)? }
             )+

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
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __token_inner {
    (@atomic [$name:ident] [$var:ident] [$($field:ty)?];) => {};
    (@atomic [$name:ident] [$var:ident] [$($field:ty)?]; $atomic:ident) => {
        #[allow(dead_code)]
        #[inline]
        pub fn $atomic<'a, I>() -> impl somen::parser::Parser<I, Output = ($($field)?)> + 'a
        where
            I: Positioned<Ok=$name> + ?Sized + 'a,
        {
            somen::parser::wrapper::Expect::new(
                somen::parser::is_some(|c|
                    $crate::__token_inner! { @match c [$name] [$var]; $($field)? }
                ),
                stringify!($atomic).into(),
            )
        }
    };
    (@parser [$name:ident] [$var:ident] [$($field:ty)?] [$src:ty] [$token:expr];) => {};
    (@parser [$name:ident] [$var:ident] [$($field:ty)?] [$src:ty] [$token:expr]; $parser:ident) => {
        #[allow(dead_code)]
        #[inline]
        pub fn $parser<'a, I>() -> impl somen::parser::Parser<I, Output = $name> + 'a
        where
            I: Input<Ok=$src> + ?Sized + 'a,
        {
            somen::parser::wrapper::Map::new(
                $token,
                $crate::__token_inner! { @closure [$name] [$var]; $($field)? },
            )
        }
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
        |inner| $name::$var(inner)
    };
}
