#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

fn after_n_unique(s: &[char], n: usize) -> Option<usize> {
    s.windows(n)
        .enumerate()
        .find(|(_i, window)| window.iter().unique().count() == n)
        .map(|(i, _window)| i + n)
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<char>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines.exactly_one()?.chars().collect())
    }

    fn part_1(
        signal: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        after_n_unique(signal, 4).ok_or(Error::ParseError)
    }

    fn part_2(
        signal: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        after_n_unique(signal, 14).ok_or(Error::ParseError)
    }
}
