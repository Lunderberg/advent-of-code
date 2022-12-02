#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

pub struct ThisDay;

impl Puzzle for ThisDay {
    const YEAR: u32 = 2022;
    const DAY: u8 = 1;
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<Vec<i32>>;
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        std::iter::from_fn(move || -> Option<Result<Vec<i32>, Error>> {
            let group: Result<Vec<_>, _> = lines
                .by_ref()
                .take_while(|line| !line.is_empty())
                .map(|line| line.parse::<i32>())
                .collect();
            match group {
                Ok(v) if v.is_empty() => None,
                Ok(v) => Some(Ok(v)),
                Err(err) => Some(Err(err.into())),
            }
        })
        .collect()
    }

    type Part1Result = i32;
    fn part_1(values: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        Ok(values
            .iter()
            .map(|elf| elf.iter().sum::<i32>())
            .max()
            .unwrap())
    }

    type Part2Result = i32;
    fn part_2(values: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        Ok(values
            .iter()
            .map(|elf| -elf.iter().sum::<i32>())
            .k_smallest(3)
            .map(|elf| -elf)
            .sum::<i32>())
    }
}
