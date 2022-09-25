//! Contains parser combinators where both parts must succeed.

use crate::common::QError;
use crate::parser::pc::*;

/// Defines a NonOptParser where all parts must succeed.
macro_rules! non_opt_seq_pc {
    ($name:ident ; $($generics:tt),+) => {
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

non_opt_seq_pc!(NonOptSeq2; A, B);
non_opt_seq_pc!(NonOptSeq3; A, B, C);
non_opt_seq_pc!(NonOptSeq4; A, B, C, D);

//
// And (with undo if the left parser supports it)
//

impl<A, B> Parser for NonOptSeq2<A, B>
where
    A: Parser,
    A::Output: Undo,
    B: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.A.parse(tokenizer)? {
            Some(left) => match self.B.parse(tokenizer)? {
                Some(right) => Ok(Some((left, right))),
                None => {
                    left.undo(tokenizer);
                    Ok(None)
                }
            },
            None => Ok(None),
        }
    }
}

pub trait AndTrait<P>
where
    Self: Sized,
{
    fn and(self, other: P) -> NonOptSeq2<Self, P>;
}

impl<S, P> AndTrait<P> for S {
    fn and(self, other: P) -> NonOptSeq2<Self, P> {
        NonOptSeq2::new(self, other)
    }
}

//
// And Demand
//

pub struct AndDemandPC<L, R>(L, R);

impl<L, R> AndDemandPC<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self(left, right)
    }
}

impl<L, R> HasOutput for AndDemandPC<L, R>
where
    L: HasOutput,
    R: HasOutput,
{
    type Output = (L::Output, R::Output);
}

impl<L, R> Parser for AndDemandPC<L, R>
where
    L: Parser,
    R: NonOptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(left) => {
                let right = self.1.parse_non_opt(tokenizer)?;
                Ok(Some((left, right)))
            }
            None => Ok(None),
        }
    }
}

impl<L, R> NonOptParser for AndDemandPC<L, R>
where
    L: NonOptParser,
    R: NonOptParser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let left = self.0.parse_non_opt(tokenizer)?;
        let right = self.1.parse_non_opt(tokenizer)?;
        Ok((left, right))
    }
}

pub trait AndDemandTrait<P>
where
    Self: Sized,
{
    fn and_demand(self, other: P) -> AndDemandPC<Self, P>;
}

impl<S, P> AndDemandTrait<P> for S {
    fn and_demand(self, other: P) -> AndDemandPC<Self, P> {
        AndDemandPC(self, other)
    }
}

pub fn seq2<A, B, F, U>(a: A, b: B, f: F) -> impl Parser<Output = U>
where
    A: Parser,
    B: NonOptParser,
    F: Fn(A::Output, B::Output) -> U,
{
    AndDemandPC::new(a, b).map(move |(x, y)| f(x, y))
}

/// A parser where if the first succeeds, all subsequent parsers must succeed.
/// The mapping function accepts all parsed arguments.
macro_rules! seq_fn {
    ($fn_name:ident ; $type_name:ident ; $($generics:tt),+) => {
        #[allow(non_snake_case)]
        pub fn $fn_name <
            P1: Parser,
            OUT,
            FM : Fn(P1::Output, $($generics::Output),+ ) -> OUT,
            $($generics: NonOptParser),+
        > (head: P1, $($generics: $generics),+, mapper: FM) -> impl Parser<Output = OUT>
        {
            AndDemandPC::new(
                head,
                $type_name::new( $($generics),+ )
            ).map(move | ( p1, ( $($generics),+ ) )| mapper(p1, $($generics),+ ) )
        }
    }
}

seq_fn!(seq3 ; NonOptSeq2 ; A, B);
seq_fn!(seq4 ; NonOptSeq3 ; A, B, C);
seq_fn!(seq5 ; NonOptSeq4 ; A, B, C, D);
