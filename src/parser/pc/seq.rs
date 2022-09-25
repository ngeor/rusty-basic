//! Contains parser combinators where given an initial optional parser,
//! the rest must succeed.

use crate::common::QError;
use crate::parser::pc::*;

/// Defines a NonOptParser where all parts must succeed.
macro_rules! non_opt_seq_pc {
    (pub struct $name:ident< $($generics:tt),+ >) => {
        #[allow(non_snake_case)]
        pub struct $name <$($generics),+> {
            // holds the parser objects
            $($generics: $generics),+
        }

        impl <$($generics),+> $name <$($generics),+> {
            #[allow(non_snake_case)]
            pub fn new($($generics: $generics),+) -> Self {
                Self {
                    $($generics),+
                }
            }
        }

        impl <$($generics),+> HasOutput for $name <$($generics),+> where $($generics : HasOutput),+ {
            type Output = ( $($generics::Output),+ );
        }

        impl <$($generics),+> NonOptParser for $name <$($generics),+> where $($generics: NonOptParser),+ {
            #[allow(non_snake_case)]
            fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
                $(
                    let $generics = self.$generics.parse_non_opt(tokenizer)?;
                )+
                Ok(
                    (
                        $($generics),+
                    )
                )
            }
        }
    };
}

non_opt_seq_pc!(pub struct NonOptSeq2<A, B>);
non_opt_seq_pc!(pub struct NonOptSeq3<A, B, C>);
non_opt_seq_pc!(pub struct NonOptSeq4<A, B, C, D>);
non_opt_seq_pc!(pub struct NonOptSeq5<A, B, C, D, E>);

// special parser implementation for NonOptSeq2,
// to implement the "and without undo" functionality

impl<L, R> Parser for NonOptSeq2<L, R>
where
    L: Parser,
    R: NonOptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.A.parse(tokenizer)? {
            Some(left) => {
                let right = self.B.parse_non_opt(tokenizer)?;
                Ok(Some((left, right)))
            }
            None => Ok(None),
        }
    }
}

pub fn seq2<A, B, F, U>(a: A, b: B, f: F) -> impl Parser<Output = U>
where
    A: Parser,
    B: NonOptParser,
    F: Fn(A::Output, B::Output) -> U,
{
    NonOptSeq2::new(a, b).map(move |(a, b)| f(a, b))
}

/// A parser where if the first succeeds, all subsequent parsers must succeed.
/// The mapping function accepts all parsed arguments.
macro_rules! seq_fn {
    ($fn_name:ident ; $type_name:ident<$($generics:tt),+>) => {
        #[allow(non_snake_case)]
        pub fn $fn_name <
            P1: Parser,
            OUT,
            FM : Fn(P1::Output, $($generics::Output),+ ) -> OUT,
            $($generics: NonOptParser),+
        > (head: P1, $($generics: $generics),+, mapper: FM) -> impl Parser<Output = OUT>
        {
            NonOptSeq2::new(
                head,
                $type_name::new( $($generics),+ )
            ).map(move | ( p1, ( $($generics),+ ) )| mapper(p1, $($generics),+ ) )
        }
    }
}

seq_fn!(seq3 ; NonOptSeq2<A, B>);
seq_fn!(seq4 ; NonOptSeq3<A, B, C>);
seq_fn!(seq5 ; NonOptSeq4<A, B, C, D>);
seq_fn!(seq6 ; NonOptSeq5<A, B, C, D, E>);
