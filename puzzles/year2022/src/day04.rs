#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput =
        Vec<(std::ops::RangeInclusive<i32>, std::ops::RangeInclusive<i32>)>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines
            .map(|line| {
                let (a, b) = line
                    .split(',')
                    .map(|elf| -> Result<_, Error> {
                        let (min, max) = elf
                            .split('-')
                            .map(|s| s.parse::<i32>())
                            .tuples()
                            .exactly_one()?;
                        Ok((min?)..=(max?))
                    })
                    .tuples()
                    .exactly_one()?;
                Ok((a?, b?))
            })
            .collect()
    }

    fn part_1(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(values
            .iter()
            .filter(|(a, b)| {
                (a.contains(b.start()) && a.contains(b.end()))
                    || (b.contains(a.start()) && (b.contains(a.end())))
            })
            .count())
    }

    fn part_2(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(values
            .iter()
            .filter(|(a, b)| a.start() <= b.end() && b.start() <= a.end())
            .count())
    }
}
