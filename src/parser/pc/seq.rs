//! Contains parser combinators where given an initial optional parser,
//! the rest must succeed.

use crate::common::QError;
use crate::parser::pc::*;

// When in opt-parser: if the first succeeds, all the rest must succeed.
// When in non-opt-parser: all parts must succeed.

macro_rules! seq_pc {
    (pub struct $name:ident<$first_type:tt, $($generic_type:tt),+ >) => {
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

        impl <$first_type, $($generic_type),+> ParserBase for $name <$first_type, $($generic_type),+>
        where
            $first_type: ParserBase,
            $($generic_type : ParserBase),+
        {
            type Output = ($first_type::Output, $($generic_type::Output),+ );
        }

        impl <$first_type, $($generic_type),+> NonOptParser for $name <$first_type, $($generic_type),+>
        where
            $first_type: NonOptParser,
            $($generic_type: NonOptParser),+
        {
            #[allow(non_snake_case)]
            fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
                let $first_type = self.$first_type.parse_non_opt(tokenizer)?;
                $(
                    let $generic_type = self.$generic_type.parse_non_opt(tokenizer)?;
                )+
                Ok(
                    (
                        $first_type,
                        $($generic_type),+
                    )
                )
            }
        }

        impl <$first_type, $($generic_type),+> OptParser for $name <$first_type, $($generic_type),+>
        where
            $first_type: OptParser,
            $($generic_type: NonOptParser),+
        {
            #[allow(non_snake_case)]
            fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
                if let Some($first_type) = self.$first_type.parse(tokenizer)? {
                    $(
                        let $generic_type = self.$generic_type.parse_non_opt(tokenizer)?;
                    )+
                    Ok(
                        Some(
                            (
                                $first_type,
                                $($generic_type),+
                            )
                        )
                    )
                } else {
                    Ok(None)
                }
            }
        }
    };
}

seq_pc!(pub struct Seq2<A, B>);
seq_pc!(pub struct Seq3<A, B, C>);
seq_pc!(pub struct Seq4<A, B, C, D>);
seq_pc!(pub struct Seq5<A, B, C, D, E>);
