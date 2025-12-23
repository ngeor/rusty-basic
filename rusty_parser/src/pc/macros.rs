//
// a struct and its `new` constructor, first parameter is implicitly a parser
//

#[macro_export]
macro_rules! parser_declaration {
    ($(#[$($attrss:tt)*])*$vis:vis struct $name: ident $(<$($generic_var_name: tt: $generic_type:tt),*>)?$({
        $($field_name: tt: $field_type: tt),*$(,)?
    })?) => {
        $vis struct $name<P$(, $($generic_type),*)?> {
            parser: P$(,
            $($generic_var_name: $generic_type),*)?$(,
            $($field_name: $field_type),*)?
        }

        $(#[$($attrss)*])*
        impl<P$(, $($generic_type),*)?> $name<P$(, $($generic_type),*)?> {
            pub fn new(parser: P$(, $($generic_var_name: $generic_type),*)?$(, $($field_name: $field_type),*)?) -> Self {
                Self {
                    parser$(,
                    $($generic_var_name),*)?$(,
                    $($field_name),*)?
                }
            }
        }
    };
}

//
// binary parser declaration, two implicit parameters:
// left: L, right: R
//

#[macro_export]
macro_rules! binary_parser_declaration {
    ($vis:vis struct $name: ident) => {
        $vis struct $name<L, R> {
            left: L,
            right: R,
        }

        impl<L, R> $name<L, R> {
            pub fn new(left: L, right: R) -> Self {
                Self { left, right }
            }
        }
    };
}

// lazy parser
// TODO store the instance one it is created, don't call factory multiple times (needs inner mutability)
#[macro_export]
macro_rules! lazy_parser {
    ($fn_vis:vis fn $fn_name:ident<I=$input_type:tt, Output=$output_type:tt> ; $struct_vis:vis struct $struct_name:ident ; $body:expr) => {
        $fn_vis fn $fn_name()  -> impl Parser<$input_type, Output=$output_type> {
            $struct_name
        }

        struct $struct_name;

        impl $struct_name {
            fn create_parser() -> impl Parser<$input_type, Output=$output_type> {
                $body
            }
        }

        impl Parser<$input_type> for $struct_name {
            type Output = $output_type;

            fn parse(&self, tokenizer: $input_type) -> ParseResult<$input_type, $output_type, $crate::error::ParseError> {
                let parser = Self::create_parser();
                parser.parse(tokenizer)
            }
        }
    };
}
