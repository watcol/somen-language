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
            pub fn parser<'__parser, __Input>() -> impl somen::parser::Parser<
                __Input,
                Output = $name $(<$($lt,)* $($T),*>)?
            > + '__parser
            where
                __Input: Input<Ok=$src> + ?Sized + '__parser,
                $(
                    $($lt: '__parser,)*
                    $($T: '__parser,)*
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
        pub fn $fname<'__parser, __Input>() -> impl somen::parser::Parser<
            __Input, Output = ($($field)?)
        > + '__parser $($(+ $lt)*)?
        where
           __Input: Positioned<Ok = Self> + ?Sized + '__parser $($(+ $lt)*)?,
            $(
                $($lt: '__parser,)*
                $($T: '__parser,)*
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
        pub fn $fname<'__parser, __Input, __Value>(inner: __Value) -> impl somen::parser::Parser<
            __Input, Output = $field
        > + '__parser $($(+ $lt)*)?
        where
            __Input: Positioned<Ok = Self> + ?Sized + '__parser $($(+ $lt)*)?,
            __Value: PartialEq<$field> + '__parser $($(+ $lt)*)?,
            $(
                $($lt: '__parser,)*
                $($T: '__parser,)*
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
        pub fn $fname<'__parser, __Input>() -> impl somen::parser::Parser<
            __Input,
            Output = $name $(<$($lt,)* $($T),*>)?
        > + '__parser
        where
            __Input: Input<Ok=$src> + ?Sized + '__parser,
            $(
                $($lt: '__parser,)*
                $($T: '__parser,)*
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

/// Automatically generate a parser for infix expressions, using precedence climbing.
#[macro_export]
macro_rules! infix {
    (
        $name:ident: $output:ty;
        $($atom_val:ident : $atom:expr => $atom_ex:expr;)+
        $(
            @[$type:ident $(($($vars:tt)*))?]
            $($op:expr => $ex:expr;)+
        )*
    ) => {{
        let $name = || somen::parser::choice(($(
            somen::parser::wrapper::Map::new(
                $atom,
                |$atom_val| -> $output { $atom_ex },
            ),
        )+));

        $(
            let $name = move || $crate::__infix_inner! {
                [$type $(($($vars)*))?] $name, $output;
                $($op => $ex;)+
            };
         )*
        $name()
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __infix_inner {
    ([prefix($val:ident)] $name:ident, $output:ty;
     $($op:expr => $ex:expr;)+) => {{
        extern crate alloc;

        somen::parser::wrapper::Map::new(
            (
                somen::parser::iterable::combinator::Collect::<_, alloc::vec::Vec<_>>::new(
                    somen::parser::iterable::generator::Repeat::new(
                        somen::parser::one_of([$($op),+]),
                        ..,
                    )
                ),
                $name(),
            ),
            |(collect, init): (alloc::vec::Vec<_>, $output)| -> $output {
                collect
                    .into_iter()
                    .rev()
                    .fold(init, |$val: $output, op| match op {
                        $(
                            $op => $ex,
                         )+
                        _ => unreachable!(),
                    })
            },
        )
    }};
    ([prefix_once($val:ident)] $name:ident, $output:ty; $($op:expr => $ex:expr;)+) => {
        somen::parser::wrapper::Map::new(
            (somen::parser::combinator::Opt::new(somen::parser::one_of([$($op),+])), $name()),
            |(op, $val): (core::option::Option<_>, $output)| -> $output {
                match op {
                    $(
                        Some($op) => $ex,
                     )+
                    Some(_) => unreachable!(),
                    None => $val,
                }
            },
        )
    };
    ([postfix($val:ident)] $name:ident, $output:ty; $($op:expr => $ex:expr;)+) => {
        somen::parser::iterable::combinator::Fold::new(
            somen::parser::iterable::generator::Repeat::new(somen::parser::one_of([$($op),+]), ..),
            $name(),
            |$val: $output, op: _| match op {
                $(
                    $op => $ex,
                 )+
                _ => unreachable!(),
            },
        )
    };
    ([postfix_once($val:ident)] $name:ident, $output:ty; $($op:expr => $ex:expr;)+) => {
        somen::parser::wrapper::Map::new(
            ($name(), somen::parser::combinator::Opt::new(somen::parser::one_of([$($op),+]))),
            |($val, op): ($output, core::option::Option<_>)| -> $output {
                match op {
                    $(
                        Some($op) => $ex,
                     )+
                    Some(_) => unreachable!(),
                    None => $val,
                }
            },
        )
    };
    ([binary($rhs:ident $(: $rt:ty)? $(= $rp:expr)?, $lhs:ident $(: $lt:ty)? $(= $lp:expr)?)]
     $name:ident, $output:ty; $($op:expr => $ex:expr;)+) => {
        somen::parser::wrapper::Map::new(
            (
                $crate::__infix_inner!(@opt $name; $($rp)?),
                somen::parser::combinator::Opt::new((
                    somen::parser::one_of([$($op),+]),
                    $crate::__infix_inner!(@opt $name; $($lp)?),
                ))
            ),
            |($rhs, op): (
                $crate::__infix_inner!(@opt_ty $output; $($rt)? $(= $rp)?),
                core::option::Option<(_, $crate::__infix_inner!(@opt_ty $output; $($lt)? $(= $lp)?))>
            )| -> $output {
                match op {
                    $(
                        Some(($op, $lhs)) => $ex,
                     )+
                    Some(_) => unreachable!(),
                    None => $rhs,
                }
            },
        )
    };
    ([left($rhs:ident, $lhs:ident $(: $lt:ty)? $(= $lp:expr)?)] $name:ident, $output:ty;
     $($op:expr => $ex:expr;)+) => {
        somen::parser::iterable::combinator::Fold::new(
            somen::parser::iterable::generator::Repeat::new(
                (somen::parser::one_of([$($op),+]), $crate::__infix_inner!(@opt $name; $($lp)?)),
                ..
            ),
            $name(),
            |$rhs: $output, (op, $lhs): (_, $crate::__infix_inner!(@opt_ty $output; $($lt)? $(= $lp)?))|
                -> $output {
                match op {
                    $(
                        $op => $ex,
                     )+
                    _ => unreachable!(),
                }
            },
        )
    };
    ([right($rhs:ident $(: $rt:ty)? $(= $rp:expr)?, $lhs:ident)] $name:ident, $output:ty;
     $($op:expr => $ex:expr;)+) => {{
        extern crate alloc;

        somen::parser::wrapper::Map::new(
            (
                somen::parser::iterable::combinator::Collect::<_, alloc::vec::Vec<_>>::new(
                    somen::parser::iterable::generator::Repeat::new(
                        (
                            $crate::__infix_inner!(@opt $name; $($rp)?),
                            somen::parser::one_of([$($op),+]),
                        ),
                        ..,
                    )
                ),
                $name(),
            ),
            |(collect, init): (
                alloc::vec::Vec<($crate::__infix_inner!(@opt_ty $output; $($rt)? $(= $rp)?), _)>,
                $output
            )| -> $output {
                collect
                    .into_iter()
                    .rev()
                    .fold(init, |$lhs, ($rhs, op)| match op {
                        $(
                            $op => $ex,
                         )+
                        _ => unreachable!(),
                    })
            },
        )
    }};
    (@opt $name:ident;) => { $name() };
    (@opt $name:ident; $parser:expr) => { $parser };
    (@opt_ty $output:ty;) => { $output };
    (@opt_ty $output:ty; = $parser:expr) => { _ };
    (@opt_ty $output:ty; $type:ty $(= $parser:expr)?) => { $type };
}
