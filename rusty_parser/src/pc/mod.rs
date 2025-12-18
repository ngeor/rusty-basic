//! base module contains functionality that can be extracted into a library,
//! it's generic and not specific to QBasic
mod accumulate;
mod allow_default;
mod allow_none;
mod allow_none_if;
mod and;
mod any;
mod chain;
mod delimited;
mod filter;
mod flat_map;
mod iif;
#[cfg(debug_assertions)]
mod logging;
mod loop_while;
mod macros;
mod many;
mod map;
mod map_err;
mod opt_and;
mod opt_zip;
mod or;
mod parse_result;
mod parsers;
mod row_col_view;
mod seq;
mod string_view;
mod surround;
mod tokenizers;
mod undo;

pub use accumulate::*;
pub use allow_default::*;
pub use allow_none::*;
pub use allow_none_if::*;
pub use any::*;
pub use chain::*;
pub use delimited::*;
pub use filter::*;
pub use iif::*;
#[cfg(debug_assertions)]
pub use logging::*;
pub use loop_while::*;
pub use map_err::*;
pub use opt_and::*;
pub use opt_zip::*;
pub use or::*;
pub use parse_result::*;
pub use parsers::*;
pub use seq::*;
pub use string_view::RcStringView;
pub use surround::*;
pub use tokenizers::*;
pub use undo::*;
