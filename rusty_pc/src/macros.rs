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

/// Parser combinator declaration, including trait that implements one function,
/// and the implementing struct with its fields and constructor.
///
/// Example
///
/// ```ignore
/// parser1_decl!(
///     trait FilterMap where Self::Error: Default {
///         fn filter_map<F, U>(predicate: F)
///         where
///             F: Fn(&Self::Output) -> Option;
///     }
///
///     struct FilterMapParser<F>;
/// );
/// ```
#[macro_export]
macro_rules! parser1 {
    (
        // comments on the trait
        $(#[$($attrss:tt)*])*
        trait $trait_name: ident
        // constraints on the trait (Self is the Parser)
        $(where $($trait_constraint:ty : $trait_bound:path),+ $(,)? )? {
            // trait function that creates a new parser
            fn $fn_name: ident
            // generic parameters to the function
            // some of them might appear in the strut fields,
            // some might not!
            $(< $($fn_generic_type:ident),+ > )?
            (
                // function arguments
                $($fn_arg_name:ident: $fn_arg_type:ty),*
            )
            // constraints on the function
            $(where $($fn_constraint:ty : $fn_bound:path),+ )?

            ;
        }

        // implementing the Parser trait for the struct created

        impl
        Parser
        for
        $struct_name: ident
        // generic types of the struct
        $(< $($struct_generic_type:ident),+ > )?
        // overall constraints (use P:: instead of Self:: here)
        $(where $($constraint:ty : $bound:path),+ )? {

            type Output = $output:ty;

            fn parse(&$self:ident, $input:ident)
            $body:block
        }
    ) => {
        // trait definition

        $(#[$($attrss)*])*
        pub trait $trait_name<I, C> : Parser<I, C>
        where
            Self: Sized, $( $($trait_constraint : $trait_bound),+ )?
        {
            fn $fn_name
            $(< $($fn_generic_type),* >)?
            (
                self
                $(, $fn_arg_name: $fn_arg_type),*
            )
            ->
            $struct_name<Self$(, $($struct_generic_type),+ )?>
            $(where $($fn_constraint : $fn_bound),+ )? {
                $struct_name::new(
                    self
                    $(, $fn_arg_name),*
                )
            }
        }

        // blanket implementation for any Parser

        impl<I, C, P> $trait_name<I, C> for P
        where
            P: Parser<I, C>,
            $( $($trait_constraint : $trait_bound),+ )? {}

        // struct

        pub struct $struct_name<P $(, $($struct_generic_type),+ )?> {
            parser: P
            $(, $fn_arg_name: $fn_arg_type)*
        }

        // constructor
        impl<P $(, $($struct_generic_type),+ )?>
        $struct_name<P $(, $($struct_generic_type),+ )?> {
            pub fn new(
                parser: P
                $(, $fn_arg_name: $fn_arg_type)*
            ) -> Self {
                Self { parser $(, $fn_arg_name)* }
            }
        }

        // implementation of the Parser trait

        impl<
            I,
            C,
            P
            $(, $($fn_generic_type),+ )?
        > Parser<I, C> for
        $struct_name<P$(, $($struct_generic_type),+ )?>
        where
            P: Parser<I, C>,
            $( $($constraint : $bound),+ )?
        {
            type Output = $output;
            type Error = P::Error;

            fn parse(&$self, $input: I) -> ParseResult<I, Self::Output, Self::Error>
                $body
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
