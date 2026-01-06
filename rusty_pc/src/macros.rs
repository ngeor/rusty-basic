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

        impl<C, L, R> $crate::SetContext<C> for $name<L, R>
        where
            C: Clone,
            L: $crate::SetContext<C>,
            R: $crate::SetContext<C>
        {
            fn set_context(&mut self, ctx: C) {
                self.left.set_context(ctx.clone());
                self.right.set_context(ctx);
            }
        }
    };
}

#[macro_export]
macro_rules! parser_combinator {
    (
        trait $trait:ident
        $(where
            $(I: $input_bound:path,)?
            $(Output: $output_bound:path,)?
            $(Error: $error_bound:path,)?
        )? {
            $(#[$($fn_attrs:tt)*])*
            fn $fn:ident
            $( < $($fn_generic_type:ident),+ > )?
            (
                $($fn_arg_name:ident: $fn_arg_type:ty),*
            )
            $(where $($fn_constraint:ty : $fn_bound:path),+ )?
            ;
        }

        struct $struct:ident
        $( < $($struct_generic_type:ident),+ > )?
        ;

        fn parse
        $( < $($parse_fn_generic_type:ident),+ > )?
        (&$self:ident, $input:ident)
        $(where $($impl_fn_constraint:ty : $impl_fn_bound:path),+ )?
            $block:block
    ) => {
        parser_combinator!(
            trait $trait
            $(where
                $(I: $input_bound,)?
                $(Output: $output_bound,)?
                $(Error: $error_bound,)?
            )? {
                $(#[$($fn_attrs)*])*
                fn $fn
                $(<$($fn_generic_type),+>)?
                (
                    $($fn_arg_name: $fn_arg_type),*
                ) -> Self::Output
                $(where $($fn_constraint : $fn_bound),+ )?
                ;
            }

            struct $struct
            $(<$($struct_generic_type),+>)?
            ;

            fn parse
            $( < $($parse_fn_generic_type),+ > )?
            (&$self, $input) -> P::Output
            $(where $($impl_fn_constraint : $impl_fn_bound ),+ )?
                $block
        );
    };
    // same as above but allows to specify the Output type
    // need to be different on the trait level e.g. Output=Option<Self::Output>
    // and different on the struct level e.g. Output=Option<P::Output>
    (
        trait $trait:ident
        $(where
            $(I: $input_bound:path,)?
            $(Output: $output_bound:path,)?
            $(Error: $error_bound:path,)?
        )? {
            $(#[$($fn_attrs:tt)*])*
            fn $fn:ident
            $( < $($fn_generic_type:ident),+ > )?
            (
                $($fn_arg_name:ident: $fn_arg_type:ty),*
            ) -> $output_type_trait:ty
            $(where $($fn_constraint:ty : $fn_bound:path),+ )?
            ;
        }

        struct $struct:ident
        $( < $($struct_generic_type:ident),+ > )?
        ;

        fn parse
        $( < $($parse_fn_generic_type:ident),+ > )?
        (&$self:ident, $input:ident) -> $output_type_struct:ty
            $(where $($impl_fn_constraint:ty : $impl_fn_bound:path),+ )?
            $block:block
    ) => {

        pub trait $trait<I, C> : Parser<I, C>
        where
            Self: Sized,
            $(
                $(I: $input_bound,)?
                $(Self::Output: $output_bound,)?
                $(Self::Error: $error_bound,)?
            )? {

            $(#[$($fn_attrs)*])*
            fn $fn
            $( < $($fn_generic_type),+ > )?
            (
                self,
                $($fn_arg_name: $fn_arg_type),*
            ) -> impl Parser<I, C, Output = $output_type_trait, Error = Self::Error>
            $(where $($fn_constraint : $fn_bound),+ )?
            {
                $struct::new(
                    self,
                    $($fn_arg_name),*
                )
            }
        }

        // blanket implementation for any Parser

        impl <I, C, P> $trait<I, C> for P
        where
            P: Parser<I, C>,
            $(
                $(I: $input_bound,)?
                $(P::Output: $output_bound,)?
                $(P::Error: $error_bound,)?
            )?
        {
        }

        struct $struct
        <P, $($($struct_generic_type),+)?> {
            parser: P,
            $($fn_arg_name: $fn_arg_type),*
        }

        impl<P, $($($struct_generic_type),+)?> $struct<P, $($($struct_generic_type),+)?> {
            pub fn new(
                parser: P,
                $($fn_arg_name: $fn_arg_type),*
            ) -> Self {
                Self {
                    parser,
                    $($fn_arg_name),*
                }
            }
        }

        impl
        <
            I,
            C,
            P
            $(, $($struct_generic_type),+)?
            $(, $($parse_fn_generic_type),+)?
        > Parser<I, C> for $struct<P, $($($struct_generic_type),+)?>
        where
            P: Parser<I, C>,
            $(
                $(I: $input_bound,)?
                $(P::Output: $output_bound,)?
                $(P::Error: $error_bound,)?
            )?
            $( $($impl_fn_constraint : $impl_fn_bound ),+ )?

        {
            type Output = $output_type_struct;
            type Error = P::Error;

            fn parse(&$self, $input: I) -> ParseResult<I, Self::Output, Self::Error>
                $block
        }

        impl
        <
            C,
            P
            $(, $($struct_generic_type),+)?
        > $crate::SetContext<C> for $struct<P, $($($struct_generic_type),+)?>
        where
            P: $crate::SetContext<C>,
        {
            fn set_context(&mut self, ctx: C)
            {
                self.parser.set_context(ctx)
            }
        }
    };
}
