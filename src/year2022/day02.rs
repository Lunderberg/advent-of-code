#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

pub struct ThisDay;

pub struct StrategyGuide {
    opponent: i32,
    col2: i32,
}

impl StrategyGuide {
    fn part1_score(&self) -> i32 {
        let player = self.col2;
        let outcome = (player - self.opponent + 1).wrapping_rem_euclid(3);
        (player + 1) + 3 * outcome
    }

    fn part2_score(&self) -> i32 {
        let outcome = self.col2;
        let player = (outcome + self.opponent - 1).wrapping_rem_euclid(3);
        (player + 1) + 3 * outcome
    }
}

impl std::str::FromStr for StrategyGuide {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        let opponent = match chars.next().ok_or(Error::NoneError)? {
            'A' => Ok(0),
            'B' => Ok(1),
            'C' => Ok(2),
            c => Err(Error::UnknownChar(c)),
        }?;

        chars.next();

        let col2 = match chars.next().ok_or(Error::NoneError)? {
            'X' => Ok(0),
            'Y' => Ok(1),
            'Z' => Ok(2),
            c => Err(Error::UnknownChar(c)),
        }?;

        Ok(StrategyGuide { col2, opponent })
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 2;

    type ParsedInput = Vec<StrategyGuide>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    type Part1Result = i32;
    fn part_1(values: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        Ok(values.iter().map(|guide| guide.part1_score()).sum())
    }

    type Part2Result = i32;
    fn part_2(values: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        Ok(values.iter().map(|guide| guide.part2_score()).sum())
    }
}
