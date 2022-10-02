//
// a struct and its `new` constructor, first parameter is implicitly a parser
//

#[macro_export]
macro_rules! parser_declaration {
    (pub struct $name: ident $(<$($generic_var_name: tt: $generic_type:tt),*>)?$({
        $($field_name: tt: $field_type: tt),*$(,)?
    })?) => {
        pub struct $name<P$(, $($generic_type),*)?> {
            parser: P$(,
            $($generic_var_name: $generic_type),*)?$(,
            $($field_name: $field_type),*)?
        }

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
    (pub struct $name: ident) => {
        pub struct $name<L, R> {
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
