//! Contains parser combinators where given an initial optional parser,
//! the rest must succeed.

use crate::*;

// When in opt-parser: if the first succeeds, all the rest must succeed.
// When in non-opt-parser: all parts must succeed.

macro_rules! seq_pc {
    (pub struct $name:ident<$first_type:tt, $($generic_type:tt),+ > ; fn $map_fn_name:ident) => {
        #[allow(non_snake_case)]
        pub struct $name <_I, _E, $first_type, $($generic_type),+> {
            // holds the first parser object (might be opt-parser or non-opt parser)
            $first_type: Box<dyn Parser<_I, Output = $first_type, Error = _E>>,
            // holds the remaining parser objects (must be non-opt parsers)
            $($generic_type: Box<dyn Parser<_I, Output = $generic_type, Error = _E>>),+
        }

        impl <_I, _E, $first_type, $($generic_type),+> $name <_I, _E, $first_type, $($generic_type),+> {
            #[allow(non_snake_case)]
            pub fn new(
                $first_type: impl Parser<_I, Output = $first_type, Error = _E> + 'static,
                $($generic_type: impl Parser<_I, Output = $generic_type, Error = _E> +'static ),+) -> Self {
                Self {
                    $first_type : Box::new($first_type),
                    $($generic_type : Box::new($generic_type) ),+
                }
            }
        }

        impl <_I, _E, $first_type, $($generic_type),+> Parser<_I> for $name <_I, _E, $first_type, $($generic_type),+>
        // where
        //     $first_type: Parser<I>,
        //     $($generic_type: Parser<I>),+
        {
            type Output = ($first_type, $($generic_type),+ );
            type Error = _E;

            #[allow(non_snake_case)]
            fn parse(&self, tokenizer: _I) -> ParseResult<_I, Self::Output, _E> {
                // the first is allowed to return incomplete
                let (tokenizer, $first_type) = self.$first_type.parse(tokenizer)?;

                $(
                    // but the rest are not...
                    let (tokenizer, $generic_type) = match self.$generic_type.parse(tokenizer) {
                        Ok(x) => x,
                        // ... so convert any error to fatal
                        Err((_, input, err)) => return Err((true, input, err)),
                    };
                )+
                Ok(
                    (
                        tokenizer,
                        (
                            $first_type,
                            $($generic_type),+
                        )
                    )
                )
            }
        }

        #[allow(non_snake_case)]
        pub fn $map_fn_name<_I, _E, $first_type, $($generic_type),+, _F, _O>(
            $first_type: impl Parser<_I, Output = $first_type, Error = _E> + 'static ,
            $($generic_type: impl Parser<_I, Output = $generic_type, Error = _E> + 'static ),+,
            mapper: _F
        ) -> impl Parser<_I, Output = _O, Error = _E>
        where
            _F: Fn($first_type, $($generic_type),+) -> _O
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
seq_pc!(pub struct Seq5<A, B, C, D, E> ; fn seq5);
seq_pc!(pub struct Seq6<A, B, C, D, E, F> ; fn seq6);
