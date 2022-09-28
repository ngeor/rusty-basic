mod dim;
mod dim_rules;
mod redim;
mod validation;

use self::dim_rules::{convert_array_dimensions, no_implicits, resolve_string_length};

pub use self::dim_rules::{on_dim, on_params};
