use crate::common::{ParserErrorTrait, QError};
use crate::parser::pc::*;

macro_rules! alt_pc {
    ($name:ident ; $($generics:tt),+) => {

        #[allow(non_snake_case)]
        pub struct $name <OUT, $($generics),+> {
            _output_type: std::marker::PhantomData<OUT>,

            $($generics: $generics),+
        }

        impl <OUT, $($generics),+> $name <OUT, $($generics),+> {
            #[allow(non_snake_case)]
            pub fn new(
                $($generics: $generics),+
            ) -> Self {
                Self {
                    _output_type: std::marker::PhantomData,
                    $($generics),+
                }
            }
        }

        // It would be nice to have a last_type, so that the last return statement is just invoking the last parser,
        // but then Rust gives an error:
        // local ambiguity when calling macro `alt_pc`: multiple parsing options: built-in NTs tt ('last_type') or tt ('generics')
        impl <OUT, $($generics : Parser<Output=OUT>),+> Parser for $name <OUT, $($generics),+> {
            type Output = OUT;
            fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<OUT, QError> {
                $(
                    let result = self.$generics.parse(tokenizer);
                    match result {
                        Err(err) if err.is_incomplete() => {
                            // continue to the next parser if incomplete
                        },
                        _ => {
                            // return on success or fatal error
                            return result;
                        }
                    }
                )+
                Err(QError::Incomplete)
            }
        }

        impl <OUT, $($generics : ParserOnce<Output=OUT>),+> ParserOnce for $name <OUT, $($generics),+> {
            type Output = OUT;
            fn parse(self, tokenizer: &mut impl Tokenizer) -> Result<OUT, QError> {
                $(
                    let result = self.$generics.parse(tokenizer);
                    match result {
                        Err(err) if err.is_incomplete() => {
                            // continue to the next parser if incomplete
                        },
                        _ => {
                            // return on success or fatal error
                            return result;
                        }
                    }
                )+
                Err(QError::Incomplete)
            }
        }
    }
}

// if the last parser is NonOpt, the Alt2 parser is also NonOpt
impl<OUT, L: Parser<Output = OUT>, R: NonOptParser<Output = OUT>> NonOptParser for Alt2<OUT, L, R> {}

alt_pc!(
    Alt2 ; A, B
);
alt_pc!(
    Alt3 ; A, B, C
);
alt_pc!(
    Alt4 ; A, B, C, D
);
alt_pc!(
    Alt5 ; A, B, C, D, E
);
alt_pc!(
    Alt6 ; A, B, C, D, E, F
);
alt_pc!(
    Alt7 ; A, B, C, D, E, F, G
);
alt_pc!(
    Alt8 ; A, B, C, D, E, F, G, H
);
alt_pc!(
    Alt9 ; A, B, C, D, E, F, G, H, I
);
alt_pc!(
    Alt10 ; A, B, C, D, E, F, G, H, I, J
);
alt_pc!(
    Alt11 ; A, B, C, D, E, F, G, H, I, J, K
);
alt_pc!(
    Alt12 ; A, B, C, D, E, F, G, H, I, J, K, L
);
alt_pc!(
    Alt13 ; A, B, C, D, E, F, G, H, I, J, K, L, M
);
alt_pc!(
    Alt14 ; A, B, C, D, E, F, G, H, I, J, K, L, M, N
);
alt_pc!(
    Alt15 ; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O
);
alt_pc!(
    Alt16 ; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P
);
