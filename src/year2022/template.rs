#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 1;

    type ParsedInput = ();
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Err(Error::NotYetImplemented)
    }

    type Part1Result = ();
    fn part_1(values: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        Err(Error::NotYetImplemented)
    }

    type Part2Result = ();
    fn part_2(values: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        Err(Error::NotYetImplemented)
    }
}