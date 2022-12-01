#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

pub struct ThisPuzzle;

impl Puzzle for ThisPuzzle {
    const YEAR: u32 = 2022;
    const DAY: u8 = 1;
    const IMPLEMENTED: bool = false;
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<i32>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        // lines
        //     .map(|line| line.parse::<i32>().map_err(|err| err.into()))
        //     .collect()
        todo!()
    }

    type Part1Result = usize;
    fn part_1(values: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        todo!()
    }

    type Part2Result = usize;
    fn part_2(values: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        todo!()
    }
}
