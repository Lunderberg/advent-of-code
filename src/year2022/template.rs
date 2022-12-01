#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

pub struct ThisPuzzle;

impl Puzzle for ThisPuzzle {
    const IMPLEMENTED: bool = true;
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 1;

    type ParsedInput = ();
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        todo!()
    }

    type Part1Result = ();
    fn part_1(values: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        todo!()
    }

    type Part2Result = ();
    fn part_2(values: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        todo!()
    }
}
