use crate::common::QError;
use crate::parser::pc::*;

//
// NonOptDelimitedPC
//

pub struct NonOptDelimitedPC<A, B> {
    parser: A,
    delimiter: B,
}

impl<A, B> HasOutput for NonOptDelimitedPC<A, B>
where
    A: HasOutput,
{
    type Output = Vec<A::Output>;
}

impl<A, B> NonOptParser for NonOptDelimitedPC<A, B>
where
    A: NonOptParser,
    B: Parser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<A::Output> = vec![];
        let mut has_more = true;
        while has_more {
            let next = self.parser.parse_non_opt(tokenizer)?;
            result.push(next);
            has_more = self.delimiter.parse(tokenizer)?.is_some();
        }
        Ok(result)
    }
}

//
// DelimitedPC
//

pub struct DelimitedPC<A, B> {
    parser: A,
    delimiter: B,
    trailing_error: QError,
}

impl<A, B> HasOutput for DelimitedPC<A, B>
where
    A: HasOutput,
{
    type Output = Vec<A::Output>;
}

// reject zero elements

impl<A, B> Parser for DelimitedPC<A, B>
where
    A: Parser,
    B: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.parser.parse(tokenizer)? {
            Some(first) => self.after_element(tokenizer, vec![first]).map(Some),
            None => Ok(None),
        }
    }
}

// allow zero elements

impl<A, B> NonOptParser for DelimitedPC<A, B>
where
    A: Parser,
    B: Parser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self.parser.parse(tokenizer)? {
            Some(first) => self.after_element(tokenizer, vec![first]),
            None => Ok(vec![]),
        }
    }
}

impl<A, B> DelimitedPC<A, B>
where
    A: Parser,
    B: Parser,
{
    fn after_element(
        &self,
        tokenizer: &mut impl Tokenizer,
        collected: Vec<A::Output>,
    ) -> Result<Vec<A::Output>, QError> {
        match self.delimiter.parse(tokenizer)? {
            Some(_) => self.after_delimiter(tokenizer, collected),
            None => Ok(collected),
        }
    }

    fn after_delimiter(
        &self,
        tokenizer: &mut impl Tokenizer,
        mut collected: Vec<A::Output>,
    ) -> Result<Vec<A::Output>, QError> {
        match self.parser.parse(tokenizer)? {
            Some(next) => {
                collected.push(next);
                self.after_element(tokenizer, collected)
            }
            None => Err(self.trailing_error.clone()),
        }
    }
}

// allow missing elements between delimiters

pub struct DelimitedAllowMissingPC<A, B> {
    parser: A,
    delimiter: B,
}

impl<A, B> HasOutput for DelimitedAllowMissingPC<A, B>
where
    A: HasOutput,
{
    type Output = Vec<Option<A::Output>>;
}

enum ListItem<A> {
    Empty,
    FinalItem(A, Box<ListItem<A>>),
    Delimited(Option<A>, Box<ListItem<A>>),
}

impl<A, B> NonOptParser for DelimitedAllowMissingPC<A, B>
where
    A: Parser,
    B: Parser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut item = ListItem::Empty;
        loop {
            let opt_value = self.parser.parse(tokenizer)?;
            match self.delimiter.parse(tokenizer)? {
                Some(_) => {
                    item = ListItem::Delimited(opt_value, Box::new(item));
                }
                None => {
                    if let Some(value) = opt_value {
                        item = ListItem::FinalItem(value, Box::new(item));
                    }
                    break;
                }
            }
        }

        let mut result: Vec<Option<A::Output>> = vec![];
        loop {
            match item {
                ListItem::Empty => {
                    break;
                }
                ListItem::FinalItem(value, box_next) => {
                    result.insert(0, Some(value));
                    item = *box_next;
                }
                ListItem::Delimited(opt_value, box_next) => {
                    if result.is_empty() {
                        return Err(QError::syntax_error("Error: trailing comma"));
                    }
                    result.insert(0, opt_value);
                    item = *box_next;
                }
            }
        }
        Ok(result)
    }
}

pub trait DelimitedTrait<P>
where
    Self: Sized,
{
    fn one_or_more_delimited_by(self, delimiter: P, trailing_error: QError)
        -> DelimitedPC<Self, P>;

    fn one_or_more_delimited_by_allow_missing(
        self,
        delimiter: P,
    ) -> DelimitedAllowMissingPC<Self, P>;

    fn one_or_more_delimited_by_non_opt(self, delimiter: P) -> NonOptDelimitedPC<Self, P>;
}

impl<S, P> DelimitedTrait<P> for S {
    fn one_or_more_delimited_by(
        self,
        delimiter: P,
        trailing_error: QError,
    ) -> DelimitedPC<Self, P> {
        DelimitedPC {
            parser: self,
            delimiter,
            trailing_error,
        }
    }

    fn one_or_more_delimited_by_allow_missing(
        self,
        delimiter: P,
    ) -> DelimitedAllowMissingPC<Self, P> {
        DelimitedAllowMissingPC {
            parser: self,
            delimiter,
        }
    }

    fn one_or_more_delimited_by_non_opt(self, delimiter: P) -> NonOptDelimitedPC<Self, P> {
        NonOptDelimitedPC {
            parser: self,
            delimiter,
        }
    }
}