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
        LParen('('),
        RParen(')'),
        Colon,
        Semicolon(';'),
        Comma(','),
        SingleQuote,
        DoubleQuote,
        Dot,
        Equals,
        Greater,
        Less,
        GreaterEquals,
        LessEquals,
        NotEquals,
        Plus,
        Minus,
        Star,
        Slash,
        Ampersand,
        ExclamationMark,
        Pound,
        DollarSign,
        Percent,
        // keyword needs to be before Identifier, because the first one wins
        Keyword,
        // Starts with letter, continues with letters or digits.
        Identifier,
        OctDigits,
        HexDigits,

        // unknown must be last
        Unknown,
    }

    const ALL_TOKEN_TYPES;
);

impl TokenType {
    pub fn matches(&self, token: &Token) -> bool {
        *self == Self::from_token(token)
    }

    pub fn from_token(token: &Token) -> Self {
        Self::from_index(token.kind())
    }
}
