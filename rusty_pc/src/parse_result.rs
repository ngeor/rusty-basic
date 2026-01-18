use crate::ParserErrorTrait;

/// Creates a failed result containing the default parse error (soft).
pub fn default_parse_error<O, E>() -> Result<O, E>
where
    E: ParserErrorTrait,
{
    Err(E::default())
}
