use rusty_common::{AtPos, HasPos, Positioned};
use rusty_pc::{ParseResult, ParseResultTrait, Parser, parser1_decl, parser1_impl};

parser1_decl!(
    trait WithPos
    where
        I: HasPos,
    {
        fn with_pos();
    }

    struct WithPosMapper;
);

parser1_impl!(
    impl Parser for WithPosMapper where I : HasPos {
        type Output = Positioned<P::Output>;

        fn parse(&self, tokenizer) {
            let pos = tokenizer.pos();
            self.parser.parse(tokenizer).map_ok(|x| x.at_pos(pos))
        }
    }
);
