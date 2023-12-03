use aoc_utils::prelude::*;

use std::collections::HashSet;

pub struct RuckSack {
    contents: Vec<u32>,
}

impl RuckSack {
    fn in_both_compartments(&self) -> impl Iterator<Item = u32> {
        let n = self.contents.len() / 2;
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

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<RuckSack>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(values
            .iter()
            .flat_map(|rucksack| rucksack.in_both_compartments())
            .sum::<u32>())
    }

    fn part_2(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        values
            .chunks(3)
            .map(|group| -> Result<u32, Error> {
                group
                    .iter()
                    .map(|rucksack| -> HashSet<_> {
                        rucksack.contents.iter().copied().collect()
                    })
                    .reduce(|a, b| a.intersection(&b).copied().collect())
                    .unwrap()
                    .into_iter()
                    .exactly_one_or_err()
            })
            .sum::<Result<u32, Error>>()
    }
}
