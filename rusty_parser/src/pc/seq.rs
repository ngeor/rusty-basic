//! Contains parser combinators where given an initial optional parser,
//! the rest must succeed.

use crate::pc::*;

// When in opt-parser: if the first succeeds, all the rest must succeed.
// When in non-opt-parser: all parts must succeed.

macro_rules! seq_pc {
    (pub struct $name:ident<$first_type:tt, $($generic_type:tt),+ > ; fn $map_fn_name:ident) => {
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

        impl <I: Tokenizer + 'static, $first_type, $($generic_type),+> Parser<I> for $name <$first_type, $($generic_type),+>
        where
            $first_type: Parser<I>,
            $($generic_type: Parser<I>),+
        {
            type Output = ($first_type::Output, $(<$generic_type as Parser<I>>::Output),+ );

            #[allow(non_snake_case)]
            fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, $crate::ParseError> {
                // the first is allowed to return incomplete
                let $first_type = match self.$first_type.parse(tokenizer) {
                    ParseResult::Ok(x) => x,
                    ParseResult::None => return ParseResult::None,
                    ParseResult::Expected(s) => return ParseResult::Expected(s),
                    ParseResult::Err(err) => return ParseResult::Err(err),
                };

                $(
                    // but the rest are not
                    let $generic_type = match self.$generic_type.parse(tokenizer) {
                        ParseResult::Ok(x) => x,
                        ParseResult::None => return ParseResult::Err($crate::ParseError::syntax_error("Could not parse")),
                        ParseResult::Expected(s) => return ParseResult::Err($crate::ParseError::SyntaxError(s)),
                        ParseResult::Err(err) => return ParseResult::Err(err),
                    };
                )+
                ParseResult::Ok(
                    (
                        $first_type,
                        $($generic_type),+
                    )
                )
            }
        }

        #[allow(non_snake_case)]
        pub fn $map_fn_name<I: Tokenizer + 'static, $first_type, $($generic_type),+, _F, _O>(
            $first_type: $first_type, $($generic_type: $generic_type),+, mapper: _F
        ) -> impl Parser<I, Output = _O>
        where
            $first_type: Parser<I>,
            $($generic_type: Parser<I>),+,
            _F: Fn($first_type::Output, $(<$generic_type as Parser<I>>::Output),+) -> _O
        {
            $name::new(
                $first_type,
                $($generic_type),+
            ).map(
                move |($first_type, $($generic_type),+)| mapper($first_type, $($generic_type),+)
            )
        }
    };

    (pub struct $name:ident<$first_type:tt, $($generic_type:tt),+ > ; fn $map_fn_name:ident ; fn $map_fn_name_non_opt:ident) => {
        seq_pc!(pub struct $name<$first_type, $($generic_type),+> ; fn $map_fn_name);

        #[allow(non_snake_case)]
        pub fn $map_fn_name_non_opt<I: Tokenizer + 'static, $first_type, $($generic_type),+, _F, _O>(
            $first_type: $first_type, $($generic_type: $generic_type),+, mapper: _F
        ) -> impl Parser<I, Output = _O>
        where
            $first_type: Parser<I>,
            $($generic_type: Parser<I>),+,
            _F: Fn(<$first_type as Parser<I>>::Output, $(<$generic_type as Parser<I>>::Output),+) -> _O
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

seq_pc!(pub struct Seq2<A, B> ; fn seq2);
seq_pc!(pub struct Seq3<A, B, C> ; fn seq3);
seq_pc!(pub struct Seq4<A, B, C, D> ; fn seq4);
seq_pc!(pub struct Seq5<A, B, C, D, E> ; fn seq5; fn seq5_non_opt);
seq_pc!(pub struct Seq6<A, B, C, D, E, F> ; fn seq6);
