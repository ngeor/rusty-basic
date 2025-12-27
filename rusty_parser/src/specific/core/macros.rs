macro_rules! bi_tuple {
    ($(#[$($attrss:tt)*])* $name: ident($left: ty, $right: ty)) => {
        $(#[$($attrss)*])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name($left, $right);

        impl $name {
            pub fn new(left: $left, right: $right) -> Self {
                Self(left, right)
            }

            pub fn try_map_right<F, E>(self, f: F) -> Result<Self, E>
            where F : FnOnce($right) -> Result<$right, E>
            {
                let (left, right) = self.into();
                f(right).map(|new_right| Self::new(left, new_right))
            }

            pub fn left(&self) -> &$left {
                &self.0
            }

            pub fn right(&self) -> &$right {
                &self.1
            }
        }

        impl From<$name> for ($left, $right) {
            fn from(value: $name) -> Self {
                (value.0, value.1)
            }
        }

        impl<'a> From<&'a $name> for (&'a $left, &'a $right) {
            fn from(value: &'a $name) -> Self {
                (&value.0, &value.1)
            }
        }
    };
}

pub(crate) use bi_tuple;
