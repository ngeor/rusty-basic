// Mixed type or

use crate::binary_parser_declaration;
use crate::common::QError;
use crate::parser::pc::{OptParser, ParserBase, Tokenizer};

pub enum ZipValue<L, R> {
    Left(L),
    Right(R),
    Both(L, R),
}

impl<L, R> ZipValue<L, R> {
    pub fn has_left(&self) -> bool {
        match self {
            Self::Left(_) | Self::Both(_, _) => true,
            _ => false,
        }
    }

    pub fn has_right(&self) -> bool {
        match self {
            Self::Right(_) | Self::Both(_, _) => true,
            _ => false,
        }
    }

    pub fn left(self) -> Option<L> {
        match self {
            Self::Left(left) | Self::Both(left, _) => Some(left),
            _ => None,
        }
    }
}

binary_parser_declaration!(struct OptZip);

pub fn opt_zip<L, R>(left: L, right: R) -> OptZip<L, R> {
    OptZip(left, right)
}

impl<L, R> ParserBase for OptZip<L, R>
where
    L: ParserBase,
    R: ParserBase,
{
    type Output = ZipValue<L::Output, R::Output>;
}

impl<L, R> OptParser for OptZip<L, R>
where
    L: OptParser,
    R: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let opt_left = self.0.parse(tokenizer)?;
        let opt_right = self.1.parse(tokenizer)?;
        match opt_left {
            Some(left) => match opt_right {
                Some(right) => Ok(Some(ZipValue::Both(left, right))),
                _ => Ok(Some(ZipValue::Left(left))),
            },
            None => match opt_right {
                Some(right) => Ok(Some(ZipValue::Right(right))),
                _ => Ok(None),
            },
        }
    }
}
