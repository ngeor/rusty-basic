// Mixed type or

use crate::pc::{Parser, Tokenizer};
use crate::{binary_parser_declaration, ParseError};

pub enum ZipValue<L, R> {
    Left(L),
    Right(R),
    Both(L, R),
}

impl<L, R> ZipValue<L, R> {
    #[allow(dead_code)]
    pub fn has_left(&self) -> bool {
        matches!(self, Self::Left(_) | Self::Both(_, _))
    }

    pub fn has_right(&self) -> bool {
        matches!(self, Self::Right(_) | Self::Both(_, _))
    }

    pub fn left(self) -> Option<L> {
        match self {
            Self::Left(left) | Self::Both(left, _) => Some(left),
            _ => None,
        }
    }

    pub fn right(self) -> Option<R> {
        match self {
            Self::Right(right) | Self::Both(_, right) => Some(right),
            _ => None,
        }
    }

    pub fn collect_right(items: Vec<Self>) -> Vec<R> {
        items
            .into_iter()
            .flat_map(|zip_value| zip_value.right().into_iter())
            .collect()
    }
}

binary_parser_declaration!(pub struct OptZip);

pub fn opt_zip<L, R>(left: L, right: R) -> OptZip<L, R> {
    OptZip::new(left, right)
}

impl<L, R> Parser for OptZip<L, R>
where
    L: Parser,
    R: Parser,
{
    type Output = ZipValue<L::Output, R::Output>;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        let opt_left = self.left.parse_opt(tokenizer)?;
        let opt_right = self.right.parse_opt(tokenizer)?;
        match opt_left {
            Some(left) => match opt_right {
                Some(right) => Ok(ZipValue::Both(left, right)),
                _ => Ok(ZipValue::Left(left)),
            },
            None => match opt_right {
                Some(right) => Ok(ZipValue::Right(right)),
                _ => Err(ParseError::Incomplete),
            },
        }
    }
}
