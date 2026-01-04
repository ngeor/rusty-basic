use rusty_pc::Token;

macro_rules! enum_with_index {
    ($vis:vis enum $name:tt $all_members:tt { $($member:tt $(: $friendly:literal)?),+$(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis enum $name {
            $($member),+
        }

        const $all_members : &[$name] = &[
            $($name::$member),+
        ];

        impl $name {
            // TODO optimize this!
            #[allow(unused_assignments)]
            pub fn to_index(&self) -> u8 {
                let mut result : u8 = 0;
                $(
                    if let Self::$member = self {
                        return result;
                    }
                    result += 1;
                )+
                panic!("should not happen")
            }

            pub fn from_index(needle: u8) -> Self {
                $all_members[needle as usize]
            }

            // TODO optimize this!
            pub fn to_str(&self) -> String {
                $(
                    $(
                        if let Self::$member = self {
                            return format!("{}", $friendly);
                        }
                    )?
                )+
                format!("Token of type {:?}", self)
            }
        }
    };
}

enum_with_index!(
    pub enum TokenType ALL_TOKEN_TYPES {
        Eol,
        Whitespace: "whitespace",
        Digits,
        LParen: '(',
        RParen: ')',
        Colon,
        Semicolon: ';',
        Comma: ',',
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
);

impl TokenType {
    pub fn matches(&self, token: &Token) -> bool {
        self.to_index() == token.kind()
    }

    pub fn from_token(token: &Token) -> Self {
        Self::from_index(token.kind())
    }
}
