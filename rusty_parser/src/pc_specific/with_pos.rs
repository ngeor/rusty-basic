use rusty_common::{AtPos, HasPos, Positioned};
use rusty_pc::{ParseResult, ParseResultTrait, Parser, parser_combinator};

parser_combinator!(
    trait WithPos
    where
        I: HasPos,
    {
        fn with_pos() -> Positioned<Self::Output>;
    }

    struct WithPosMapper;

    fn parse(&self, tokenizer) -> Positioned<P::Output> {
        let pos = tokenizer.pos();
        self.parser.parse(tokenizer).map_ok(|x| x.at_pos(pos))
    }
);
