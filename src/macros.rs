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
    ($(#[$attrs:meta])* $vis:vis enum $name:ident $(<
        $($lt:lifetime $(: $ltltbound:lifetime)?),* $(,)?
        $($T:ident $(: $bound:path)? $(: ?$sized:path)? $(: $ltbound:lifetime)? $(= $default:ty)?),* $(,)?
    >)? : $src:ty
    $(where $($U:ty $(: $whbound:path)? $(: ?$whsized:path)? $(: $whltbound:lifetime)?),* $(,)?)?
     {
        $(
            $(#[$f_attrs:meta])*
            $(@[$($key:ident = $value:ident),* $(,)?])?
            $var:ident $(($field:ty))? = $token:expr
        ),+ $(,)?
    }) => {
        $(#[$attrs])*
        $vis enum $name $(<
            $($lt $(: $ltltbound)?,)*
            $($T $(: $bound)? $(: ?$sized)? $(: $ltbound)? $(= $default)?),*
        >)?
        $(where $($U $(: $whbound)? $(: ?$whsized)? $(: $whltbound)?,)*)?
        {
            $($(#[$f_attrs])* $var $(($field))?,)+
        }

        impl $(<
            $($lt $(: $ltltbound)?,)*
            $($T $(: $bound)? $(: ?$sized)? $(: $ltbound)? $(= $default)?),*
        >)? $name $(<$($lt,)* $($T),*>)?
        $(where $($U $(: $whbound)? $(: ?$whsized)? $(: $whltbound)?,)*)?
        {
            #[allow(dead_code)]
            pub fn parser<'a, I>() -> impl somen::parser::Parser<
                I,
                Output = $name $(<$($lt,)* $($T),*>)?
            > + 'a
            where
                I: Input<Ok=$src> + ?Sized + 'a,
                $(
                    $($lt: 'a,)*
                    $($T: 'a,)*
                )?
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

            $crate::__token_inner! {@expand [$name] [$src]  $([$([$lt])* | $([$T])*])?;
                $([$var] [$($field)?] [$token] [|] $($([$key = $value])*)?;)+
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __token_inner {
    (@expand [$name:ident] [$src:ty]; $(
            [$var:ident] [$($field:ty)?] [$token:expr] [|] $([$key:ident = $value:ident])*;
    )*) => {
        $(
            $crate::__token_inner! {
                @method [$name] [$src] [$var] [$($field)?] [$token];
                $([$key = $value])*
            }
        )*
    };
    (@expand [$name:ident] [$src:ty] [|]; $(
            [$var:ident] [$($field:ty)?] [$token:expr] [$([$lt:lifetime])* | $([$T:ident])*]
            $([$key:ident = $value:ident])*;
    )*) => {
        $(
            $crate::__token_inner! {
                @method [$name] [$src] [$var] [$($field)?] [$token] [$($lt),*|$($T),*];
                $([$key = $value])*
            }
        )*
    };
    (@expand [$name:ident] [$src:ty] [|[$T:ident]$([$rest:ident])*]; $(
            [$var:ident] [$($field:ty)?] [$token:expr] [$([$lt:lifetime])* | $([$U:ident])*]
            $([$key:ident = $value:ident])*;
    )*) => {
        $crate::__token_inner! { @expand [$name] [$src] [|$([$rest])*]; $(
            [$var] [$($field)?] [$token] [$([$lt])*|$([$U])*[$T]] $([$key = $value])*;
        )* }
    };
    (@expand [$name:ident] [$src:ty] [[$lt:lifetime]$([$rest:lifetime])*|$([$T:ident])*]; $(
            [$var:ident] [$($field:ty)?] [$token:expr] [$([$lt2:lifetime])*|]
            $([$key:ident = $value:ident])*;
    )*) => {
        $crate::__token_inner! { @expand [$name] [$src] [$([$rest])*|$([$T])*]; $(
            [$var] [$($field)?] [$token] [$([$lt2])*[$lt]|] $([$key = $value])*;
        )* }
    };
    (@method [$name:ident] [$src:ty] [$var:ident] [$($field:ty)?] [$token:expr]
        $([$($lt:lifetime),*|$($T:ident),*])?;) => {};
    (@method [$name:ident] [$src:ty] [$var:ident] [$($field:ty)?] [$token:expr]
        $([$($lt:lifetime),*|$($T:ident),*])?; [match = $fname:ident]$([$k:ident = $v:ident])*) => {
        #[allow(dead_code)]
        #[inline]
        pub fn $fname<'a, I>() -> impl somen::parser::Parser<I, Output = ($($field)?)> + 'a $($(+ $lt)*)?
        where
           I: Positioned<Ok = Self> + ?Sized + 'a $($(+ $lt)*)?,
            $(
                $($lt: 'a,)*
                $($T: 'a,)*
            )?
        {
            somen::parser::wrapper::Expect::new(
                somen::parser::is_some(|c|
                    $crate::__token_inner! { @match c [$name] [$var]; $($field)? }
                ),
                stringify!($fname).into(),
            )
        }

        $crate::__token_inner! { @method [$name] [$src] [$var] [$($field)?] [$token] $([$($lt),*|$($T),*])?; $([$k = $v])* }
    };
    (@method [$name:ident] [$src:ty] [$var:ident] [$field:ty] [$token:expr]
        $([$($lt:lifetime),*|$($T:ident),*])?; [match_arg = $fname:ident]$([$k:ident = $v:ident])*) => {
        #[allow(dead_code)]
        #[inline]
        pub fn $fname<'a, I, T>(inner: T) -> impl somen::parser::Parser<I, Output = $field> + 'a $($(+ $lt)*)?
        where
            I: Positioned<Ok = Self> + ?Sized + 'a $($(+ $lt)*)?,
            T: PartialEq<$field> + 'a $($(+ $lt)*)?,
            $(
                $($lt: 'a,)*
                $($T: 'a,)*
            )?
        {
            somen::parser::wrapper::Expect::new(
                somen::parser::is_some(move |c| match c {
                    $name::$var(val) if inner == val => Some(val),
                    _ => None,
                }),
                stringify!($fname).into(),
            )
        }

        $crate::__token_inner! { @method [$name] [$src] [$var] [$field] [$token] $([$($lt),*|$($T),*])?; $([$k = $v])* }
    };
    (@method [$name:ident] [$src:ty] [$var:ident] [] [$token:expr]
        $([$($lt:lifetime),*|$($T:ident),*])?; [match_arg = $fname:ident]$([$k:ident = $v:ident])*) => {
        compile_error!("`match_arg` is not supported for variants without fields.");

        $crate::__token_inner! { @method [$name] [$src] [$var] [] [$token] $([$($lt),*|$($T),*])?; $([$k = $v])* }
    };
    (@method [$name:ident] [$src:ty] [$var:ident] [$($field:ty)?] [$token:expr]
        $([$($lt:lifetime),*|$($T:ident),*])?; [single = $fname:ident]$([$k:ident = $v:ident])*) => {
        #[allow(dead_code)]
        #[inline]
        pub fn $fname<'a, I>() -> impl somen::parser::Parser<I, Output = $name $(<$($lt,)* $($T),*>)?> + 'a
        where
            I: Input<Ok=$src> + ?Sized + 'a,
            $(
                $($lt: 'a,)*
                $($T: 'a,)*
            )?
        {
            somen::parser::wrapper::Map::new(
                $token,
                $crate::__token_inner! { @closure [$name] [$var]; $($field)? },
            )
        }

        $crate::__token_inner! { @method [$name] [$src] [$var] [$($field)?] [$token] $([$($lt),*|$($T),*])?; $([$k = $v])* }
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
