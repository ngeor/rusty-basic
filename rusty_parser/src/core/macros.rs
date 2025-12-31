// `bi_tuple` creates a tuple of two items.
//
// Example: `bi_tuple!(Person(name: String, age: int))`
//
// It implements the `From` trait to break apart the tuple into the members
// or take references to them.
//
// It does not implement `AsRef` (or any other trait) per item type, as it might be that the two types are the same.
macro_rules! bi_tuple {
    ($(#[$($attrss:tt)*])* $name: ident($left_name: tt: $left_type: ty, $right_name: tt: $right_type: ty)) => {
        $(#[$($attrss)*])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name($left_type, $right_type);

        impl $name {
            /// Creates a new instance of [$name].
            pub fn new($left_name: $left_type, $right_name: $right_type) -> Self {
                Self($left_name, $right_name)
            }

            /// Tries to map the right side of the tuple with the given function.
            pub fn try_map_right<F, E>(self, f: F) -> Result<Self, E>
            where F : FnOnce($right_type) -> Result<$right_type, E>
            {
                let ($left_name, $right_name) = self.into();
                f($right_name).map(|new_right| Self::new($left_name, new_right))
            }

            /// Gets a reference to the left side member of the tuple.
            pub fn $left_name(&self) -> &$left_type {
                &self.0
            }

            /// Gets a reference to the right side member of the tuple.
            pub fn $right_name(&self) -> &$right_type {
                &self.1
            }
        }

        // Break apart the tuple

        impl From<$name> for ($left_type, $right_type) {
            fn from(value: $name) -> Self {
                (value.0, value.1)
            }
        }

        // Get references to both members of the tuple

        impl<'a> From<&'a $name> for (&'a $left_type, &'a $right_type) {
            fn from(value: &'a $name) -> Self {
                (&value.0, &value.1)
            }
        }
    };
}

pub(crate) use bi_tuple;
