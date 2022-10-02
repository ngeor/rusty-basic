//! Contains parser combinators where given an initial optional parser,
//! the rest must succeed.

use crate::common::QError;
use crate::parser::pc::*;

// When in opt-parser: if the first succeeds, all the rest must succeed.
// When in non-opt-parser: all parts must succeed.

macro_rules! seq_pc {
    (pub struct $name:ident<$first_type:tt, $($generic_type:tt),+ > ; fn $map_fn_name:ident ; fn $map_fn_name_non_opt:ident) => {
        #[allow(non_snake_case)]
        pub struct $name <$first_type, $($generic_type),+> {
            // holds the first parser object (might be opt-parser or non-opt parser)
            $first_type: $first_type,
            // holds the remaining parser objects (must be non-opt parsers)
            $($generic_type: $generic_type),+
        }

        impl <$first_type, $($generic_type),+> $name <$first_type, $($generic_type),+> {
            #[allow(non_snake_case)]
            pub fn new($first_type: $first_type, $($generic_type: $generic_type),+) -> Self {
                Self {
                    $first_type,
                    $($generic_type),+
                }
            }
        }

        impl <$first_type, $($generic_type),+> Parser for $name <$first_type, $($generic_type),+>
        where
            $first_type: Parser,
            $($generic_type: Parser + NonOptParser),+
        {
            type Output = ($first_type::Output, $($generic_type::Output),+ );

            #[allow(non_snake_case)]
            fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
                // the first is allowed to return incomplete
                let $first_type = self.$first_type.parse(tokenizer)?;
                $(
                    // but the rest are not, hence `.no_incomplete()`
                    let $generic_type = self.$generic_type.parse(tokenizer)?;
                )+
                Ok(
                    (
                        $first_type,
                        $($generic_type),+
                    )
                )
            }
        }

        // if all parsers are NonOpt, the sequence is also NonOpt

        impl <$first_type, $($generic_type),+> NonOptParser for $name <$first_type, $($generic_type),+>
        where
            $first_type: Parser + NonOptParser,
            $($generic_type: Parser + NonOptParser),+
        {
        }

        #[allow(non_snake_case)]
        pub fn $map_fn_name<$first_type, $($generic_type),+, _F, _O>(
            $first_type: $first_type, $($generic_type: $generic_type),+, mapper: _F
        ) -> impl Parser<Output = _O>
        where
            $first_type: Parser,
            $($generic_type: Parser + NonOptParser),+,
            _F: Fn($first_type::Output, $($generic_type::Output),+) -> _O
        {
            $name::new(
                $first_type,
                $($generic_type),+
            ).map(
                move |($first_type, $($generic_type),+)| mapper($first_type, $($generic_type),+)
            )
        }

        #[allow(non_snake_case)]
        pub fn $map_fn_name_non_opt<$first_type, $($generic_type),+, _F, _O>(
            $first_type: $first_type, $($generic_type: $generic_type),+, mapper: _F
        ) -> impl Parser<Output = _O> + NonOptParser
        where
            $first_type: Parser + NonOptParser,
            $($generic_type: Parser + NonOptParser),+,
            _F: Fn($first_type::Output, $($generic_type::Output),+) -> _O
        {
            $name::new(
                $first_type,
                $($generic_type),+
            ).map(
                move |($first_type, $($generic_type),+)| mapper($first_type, $($generic_type),+)
            )
        }
    };
}

seq_pc!(pub struct Seq2<A, B> ; fn seq2; fn seq2_non_opt);
seq_pc!(pub struct Seq3<A, B, C> ; fn seq3; fn seq3_non_opt);
seq_pc!(pub struct Seq4<A, B, C, D> ; fn seq4; fn seq4_non_opt);
seq_pc!(pub struct Seq5<A, B, C, D, E> ; fn seq5; fn seq5_non_opt);
seq_pc!(pub struct Seq6<A, B, C, D, E, F> ; fn seq6; fn seq6_non_opt);
