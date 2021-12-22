#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

pub struct Day07;

impl Day07 {}

impl Puzzle for Day07 {
    const DAY: u8 = 7;
    const IMPLEMENTED: bool = true;
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<i64>;
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines
            .next()
            .unwrap()
            .split(',')
            .map(|s| s.parse::<i64>())
            .collect::<Result<Vec<_>, _>>()?)
    }

    type Part1Result = i64;
    fn part_1(
        crab_pos: &Self::ParsedInput,
    ) -> Result<Self::Part1Result, Error> {
        let (low, high) = match crab_pos.iter().map(|x| *x).minmax() {
            itertools::MinMaxResult::NoElements => Err(Error::CannotFindMinMax),
            itertools::MinMaxResult::OneElement(_) => {
                Err(Error::CannotFindMinMax)
            }
            itertools::MinMaxResult::MinMax(low, high) => Ok((low, high)),
        }?;

        let lowest_fuel = (low..=high)
            .map(|pos| crab_pos.iter().map(|c| (pos - c).abs()).sum::<i64>())
            .min()
            .unwrap();

        Ok(lowest_fuel)
    }

    type Part2Result = i64;
    fn part_2(
        crab_pos: &Self::ParsedInput,
    ) -> Result<Self::Part1Result, Error> {
        let (low, high) = match crab_pos.iter().map(|x| *x).minmax() {
            itertools::MinMaxResult::NoElements => Err(Error::CannotFindMinMax),
            itertools::MinMaxResult::OneElement(_) => {
                Err(Error::CannotFindMinMax)
            }
            itertools::MinMaxResult::MinMax(low, high) => Ok((low, high)),
        }?;

        let lowest_fuel = (low..=high)
            .map(|pos| {
                crab_pos
                    .iter()
                    .map(|c| {
                        let steps = (c - pos).abs();
                        steps * (steps + 1) / 2
                    })
                    .sum::<i64>()
            })
            .min()
            .unwrap();

        Ok(lowest_fuel)
    }
}
