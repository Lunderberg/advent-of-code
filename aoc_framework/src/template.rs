use aoc_utils::prelude::*;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = ();
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        todo!(
            "Parsing of input {:?} not yet implemented",
            lines.into_iter().next()
        )
    }

    fn part_1(_: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        Err::<(), _>(Error::NotYetImplemented)
    }

    fn part_2(_: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        Err::<(), _>(Error::NotYetImplemented)
    }
}
