#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

fn after_n_unique(s: &[char], n: usize) -> Option<usize> {
    s.windows(n)
        .enumerate()
        .find(|(_i, window)| window.iter().unique().count() == n)
        .map(|(i, _window)| i + n)
}

pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 6;

    type ParsedInput = Vec<char>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines.exactly_one()?.chars().collect())
    }

    type Part1Result = usize;
    fn part_1(signal: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        after_n_unique(signal, 4).ok_or(Error::ParseError)
    }

    type Part2Result = usize;
    fn part_2(signal: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        after_n_unique(signal, 14).ok_or(Error::ParseError)
    }
}
