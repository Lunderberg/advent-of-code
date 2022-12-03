#![allow(unused_imports)]
use crate::{Error, Puzzle};

use std::collections::HashSet;

use itertools::Itertools;

pub struct RuckSack {
    contents: Vec<u32>,
}

impl RuckSack {
    fn in_both_compartments(&self) -> impl Iterator<Item = u32> {
        let n = (self.contents.len() / 2) as usize;
        let first: HashSet<u32> = self.contents[..n].iter().copied().collect();
        let second: HashSet<u32> = self.contents[n..].iter().copied().collect();
        first
            .intersection(&second)
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl std::str::FromStr for RuckSack {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let contents = s
            .chars()
            .map(|c| match c {
                'a'..='z' => Ok((c as u32) - ('a' as u32) + 1),
                'A'..='Z' => Ok((c as u32) - ('A' as u32) + 27),
                _ => Err(Error::UnknownChar(c)),
            })
            .collect::<Result<_, _>>()?;
        Ok(RuckSack { contents })
    }
}

pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 3;

    type ParsedInput = Vec<RuckSack>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    type Part1Result = u32;
    fn part_1(values: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        Ok(values
            .iter()
            .flat_map(|rucksack| rucksack.in_both_compartments())
            .sum::<u32>())
    }

    type Part2Result = u32;
    fn part_2(values: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        values
            .chunks(3)
            .map(|group| -> Result<u32, Error> {
                Ok(group
                    .iter()
                    .map(|rucksack| -> HashSet<_> {
                        rucksack.contents.iter().copied().collect()
                    })
                    .reduce(|a, b| a.intersection(&b).copied().collect())
                    .unwrap()
                    .into_iter()
                    .exactly_one()?)
            })
            .sum::<Result<u32, Error>>()
    }
}
