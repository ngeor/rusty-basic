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

#[macro_export]
macro_rules! parser1 {
    (
        $(#[$($attrss:tt)*])*trait $trait_name: ident; struct $struct_name: ident; fn $fn_name: ident
    ) => {
        // trait definition

        $(#[$($attrss)*])*
        pub trait $trait_name<I, C> : Parser<I, C> where Self: Sized {
            fn $fn_name(self) -> $struct_name<Self> {
                $struct_name::new(self)
            }
        }

        // blanket implementation for any Parser

        impl<I, C, P> $trait_name<I, C> for P where P: Parser<I, C> {}

        // struct

        pub struct $struct_name<P> {
            parser: P,
        }

        // constructor
        impl<P> $struct_name<P> {
            pub fn new(parser: P) -> Self {
                Self { parser }
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
    ($fn_vis:vis fn $fn_name:ident<I=$input_type:tt, Output=$output_type:tt, Error=$error_type:tt> ; $struct_vis:vis struct $struct_name:ident ; $body:expr) => {
        $fn_vis fn $fn_name()  -> impl Parser<$input_type, Output=$output_type, Error=$error_type> {
            $struct_name
        }

        struct $struct_name;

        impl $struct_name {
            fn create_parser() -> impl Parser<$input_type, Output=$output_type, Error=$error_type> {
                $body
            }
        }

        impl Parser<$input_type> for $struct_name {
            type Output = $output_type;
            type Error = $error_type;

            fn parse(&self, tokenizer: $input_type) -> ParseResult<$input_type, $output_type, $error_type> {
                let parser = Self::create_parser();
                parser.parse(tokenizer)
            }
        }
    };
}
