#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

pub struct ThisDay;

impl Puzzle for ThisDay {
    const YEAR: u32 = 2021;
    const DAY: u8 = 1;
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<i32>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines
            .map(|line| line.parse::<i32>().map_err(|err| err.into()))
            .collect()
    }

    type Part1Result = usize;
    fn part_1(values: &Vec<i32>) -> Result<Self::Part1Result, Error> {
        Ok(values.iter().tuple_windows().filter(|(a, b)| a < b).count())
    }

    type Part2Result = usize;
    fn part_2(values: &Vec<i32>) -> Result<Self::Part2Result, Error> {
        Ok(values
            .iter()
            .tuple_windows()
            .map(|(a, b, c)| a + b + c)
            .tuple_windows()
            .filter(|(a, b)| a < b)
            .count())
    }
}
