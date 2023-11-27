#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

fn iter_hash(
    prefix: impl std::fmt::Display,
) -> impl Iterator<Item = md5::Digest> {
    std::iter::repeat(())
        .enumerate()
        .map(|(i, _)| i)
        .map(move |i| md5::compute(format!("{prefix}{i}")))
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = String;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines.exactly_one()?.to_string())
    }

    fn part_1(
        prefix: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        iter_hash(prefix)
            .enumerate()
            .find(|(_i, hash)| hash[0] == 0 && hash[1] == 0 && hash[2] < 16)
            .map(|(i, _hash)| i)
            .ok_or(Error::NoneError)
    }

    fn part_2(
        prefix: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        iter_hash(prefix)
            .enumerate()
            .find(|(_i, hash)| hash[0] == 0 && hash[1] == 0 && hash[2] == 0)
            .map(|(i, _hash)| i)
            .ok_or(Error::NoneError)
    }
}
