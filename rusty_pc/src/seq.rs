//! Contains parser combinators where given an initial optional parser,
//! the rest must succeed.

use crate::*;

macro_rules! seq_pc {
    (
        struct $name:ident
        <$first_type:tt, $($generic_type:tt),+ >
        ;
        fn $map_fn_name:ident
    ) => {

        // struct definition

        #[allow(non_snake_case)]
        struct $name
        <_I, _C, _E, _F, $first_type, $($generic_type),+>
        {
            // holds the function that will map the result
            _mapper: _F,
            // holds the first parser
            $first_type: Box<dyn Parser<_I, _C, Output = $first_type, Error = _E>>,
            // holds the remaining parsers (not allowed to return soft errors)
            $($generic_type: Box<dyn Parser<_I, _C, Output = $generic_type, Error = _E>>),+
        }

        // constructor

        impl
        <_I, _C, _E, _F, $first_type, $($generic_type),+>
        $name
        <_I, _C, _E, _F, $first_type, $($generic_type),+>
        {
            #[allow(non_snake_case)]
            pub fn new(
                _mapper: _F,
                $first_type: impl Parser<_I, _C, Output = $first_type, Error = _E> + 'static,
                $($generic_type: impl Parser<_I, _C, Output = $generic_type, Error = _E> +'static ),+) -> Self
                where _I: InputTrait {
                Self {
                    _mapper,
                    $first_type : Box::new($first_type),
                    $($generic_type : Box::new($generic_type) ),+
                }
            }
        }

        // Parser implementation

        impl
        <_I, _C, _E, _F, _O, $first_type, $($generic_type),+>
        Parser<_I, _C>
        for
        $name
        <_I, _C, _E, _F, $first_type, $($generic_type),+>
        where
            _I: $crate::InputTrait,
            _C: Clone,
            _E: $crate::ParserErrorTrait,
            _F : Fn($first_type, $($generic_type),+) -> _O
        {
            type Output = _O;
            type Error = _E;

            #[allow(non_snake_case)]
            fn parse(&mut self, tokenizer: &mut _I) -> Result<Self::Output, _E> {
                // the first is allowed to return incomplete
                let $first_type = self.$first_type.parse(tokenizer)?;

                $(
                    // but the rest are not...
                    let $generic_type = match self.$generic_type.parse(tokenizer) {
                        Ok(x) => x,
                        // ... so convert any error to fatal
                        Err(err) => return Err(err.to_fatal()),
                    };
                )+
                Ok(
                    (self._mapper)($first_type, $($generic_type),+)
                )
            }
        }

        #[allow(non_snake_case)]
        pub fn $map_fn_name<_I, _C, _E, _F, _O, $first_type, $($generic_type),+>(
            $first_type: impl Parser<_I, _C, Output = $first_type, Error = _E> + 'static ,
            $($generic_type: impl Parser<_I, _C, Output = $generic_type, Error = _E> + 'static ),+,
            mapper: _F
        ) -> impl Parser<_I, _C, Output = _O, Error = _E>
        where
            _I: $crate::InputTrait,
            _C: Clone,
            _E: $crate::ParserErrorTrait,
            _F: Fn($first_type, $($generic_type),+) -> _O
        {
            $name::new(
                mapper,
                $first_type,
                $($generic_type),+
            )
        }
    };
}

seq_pc!(struct Seq2<A, B> ; fn seq2);
seq_pc!(struct Seq3<A, B, C> ; fn seq3);
seq_pc!(struct Seq4<A, B, C, D> ; fn seq4);
seq_pc!(struct Seq5<A, B, C, D, E> ; fn seq5);
seq_pc!(struct Seq6<A, B, C, D, E, F> ; fn seq6);
