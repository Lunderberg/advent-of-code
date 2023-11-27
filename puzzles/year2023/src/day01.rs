#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

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

    fn part_1(
        _values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        todo!("Part 1 not yet implemented");
        #[allow(unreachable_code)]
        Ok(0)
    }

    fn part_2(
        _values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        todo!("Part 2 not yet implemented");
        #[allow(unreachable_code)]
        Ok(0)
    }
}
