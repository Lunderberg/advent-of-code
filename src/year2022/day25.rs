#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct SnafuNumber {
    inner: i64,
}

impl std::str::FromStr for SnafuNumber {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = s
            .chars()
            .map(|c| match c {
                '2' => Ok(2),
                '1' => Ok(1),
                '0' => Ok(0),
                '-' => Ok(-1),
                '=' => Ok(-2),
                _ => Err(Error::UnknownChar(c)),
            })
            .fold_ok(0, |a: i64, b: i64| a * 5 + b)?;
        Ok(SnafuNumber { inner })
    }
}

impl Display for SnafuNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut remainder = self.inner;
        std::iter::from_fn(|| -> Option<char> {
            (remainder != 0).then(|| {
                let digit = remainder.rem_euclid(5);
                let (digit, c) = match digit {
                    0 => (0, '0'),
                    1 => (1, '1'),
                    2 => (2, '2'),
                    3 => (-2, '='),
                    4 => (-1, '-'),
                    _ => panic!("Unknown digit: {}", digit),
                };
                remainder = (remainder - digit).div_euclid(5);
                c
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .try_for_each(|c| write!(f, "{c}"))
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<SnafuNumber>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let sum = values.iter().map(|value| value.inner).sum::<i64>();
        let snafu = SnafuNumber { inner: sum };
        let as_str = format!("{snafu}");
        Ok(as_str)
    }

    fn part_2(
        _values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(())
    }
}
