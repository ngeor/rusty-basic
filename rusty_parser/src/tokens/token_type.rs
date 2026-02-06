use rusty_pc::Token;

macro_rules! token_type_enum {
    (
        $vis:vis enum $name:ident
        {
            $(
                $member:ident $( ( $friendly:literal ) )?
            ),+
            $(,)?
        }

        const $all_members:ident;
    ) => {

        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        $vis enum $name {
            $($member),+
        }

        const $all_members : &[$name] = &[
            $($name::$member),+
        ];

        impl $name {
            pub fn get_index(&self) -> u8 {
                $all_members.binary_search(self).expect("should not happen") as u8
            }

            pub fn from_index(needle: u8) -> Self {
                $all_members[needle as usize]
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$member => token_type_enum!( $member $(, $friendly )? ).fmt(f),
                    )+
                }
            }
        }
    };

    // generate the text representation of the Token,
    // favoring the friendly name.
    ($name: ident, $friendly: literal) => { $friendly };

    // generate the text representation of the Token, using the string representation of the enum.
    ($name: ident) => { stringify!($name) };
}

token_type_enum!(
    pub enum TokenType {
        Eol,
        Whitespace("whitespace"),
        Digits,
        GreaterEquals,
        Greater,
        LessEquals,
        Less,
        Equals,
        NotEquals,
        // keyword needs to be before Identifier, because the first one wins
        Keyword,
        // Starts with letter, continues with letters or digits.
        Identifier,
        OctDigits,
        HexDigits,

        // symbol must be last
        Symbol,
    }

    const ALL_TOKEN_TYPES;
);

impl TokenType {
    pub fn from_token(token: &Token) -> Self {
        Self::from_index(token.kind())
    }
}

/// A trait that checks if the current value matches the given token.
pub trait TokenMatcher {
    /// Checks if the current value matches the given token.
    fn matches_token(&self, token: &Token) -> bool;
}

impl TokenMatcher for TokenType {
    /// Checks if the token is of this token type.
    fn matches_token(&self, token: &Token) -> bool {
        self.get_index() == token.kind()
    }
}

impl TokenMatcher for char {
    /// Checks if this is a Symbol token containing this character.
    fn matches_token(&self, token: &Token) -> bool {
        TokenType::Symbol.matches_token(token) && token.demand_single_char() == *self
    }
}
