use crate::common::QError;
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

        impl <OUT, $($generics),+> HasOutput for $name <OUT, $($generics),+> {
            type Output = OUT;
        }

        impl <OUT, $($generics : Parser<Output=OUT>),+> Parser for $name <OUT, $($generics),+> {
            fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<OUT>, QError> {
                $(
                    if let Some(value) = self.$generics.parse(tokenizer)? {
                        return Ok(Some(value));
                    }
                )+
                Ok(None)
            }
        }
    }
}

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

impl<O, L, R> NonOptParser for Alt2<O, L, R>
where
    L: Parser<Output = O>,
    R: NonOptParser<Output = O>,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self.A.parse(tokenizer)? {
            Some(left) => Ok(left),
            _ => self.B.parse_non_opt(tokenizer),
        }
    }
}

// OrTrait

pub trait OrTrait<O, P>
where
    Self: Sized + HasOutput,
{
    fn or(self, other: P) -> Alt2<O, Self, P>;
}

impl<O, S, P> OrTrait<O, P> for S
where
    S: HasOutput,
{
    fn or(self, other: P) -> Alt2<O, Self, P> {
        Alt2::new(self, other)
    }
}
