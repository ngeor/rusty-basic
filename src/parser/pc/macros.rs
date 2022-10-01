//
// a struct and its `new` constructor, first parameter is implicitly a parser
//

#[macro_export]
macro_rules! parser_declaration {
    (struct $name: ident $(<$($generic_var_name: tt: $generic_type:tt),*>)?$({
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

// TODO: use left and right instead of 0 and 1

#[macro_export]
macro_rules! binary_parser_declaration {
    (struct $name: ident $(<$($generic_var_name: tt: $generic_type:tt),*>)?) => {
        pub struct $name<L, R$(, $($generic_type),*)?>(L, R$(, $($generic_type),*)?);

        impl<L, R$(, $($generic_type),*)?> $name<L, R$(, $($generic_type),*)?> {
            pub fn new(left: L, right: R$(, $($generic_var_name: $generic_type),*)?) -> Self {
                Self(left, right$(, $($generic_var_name),*)?)
            }
        }
    };
}

//
// unary no-arg parser macro
//

#[macro_export]
macro_rules! parser_decorator {
    (struct $name: ident $(<$($generic_var_name: tt: $generic_type:tt),*>)?$({
        $($field_name: tt: $field_type: tt),*$(,)?
    })?) => {
        crate::parser_declaration!(struct $name$(<$($generic_var_name: $generic_type),*>)?$({
            $($field_name: $field_type),*
        })?);

        impl<P$(, $($generic_type),*)?> ParserBase for $name<P$(, $($generic_type),*)?>
        where
            P: ParserBase,
        {
            type Output = P::Output;
        }
    };
}
