#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

trait IterEscapeClusters {
    fn iter_clusters(&self) -> impl Iterator<Item = &str> + '_;
}

impl IterEscapeClusters for str {
    fn iter_clusters(&self) -> impl Iterator<Item = &str> + '_ {
        let mut iter = self.chars().enumerate().skip(1);

        std::iter::from_fn(move || -> Option<_> {
            let (start, c) = iter.next()?;

            if c == '"' {
                return None;
            }

            let end = if c == '\\' {
                let (esc_index, esc) = iter.next()?;
                if esc == 'x' {
                    iter.next()?;
                    iter.next()?.0
                } else {
                    esc_index
                }
            } else {
                start
            };

            self.get(start..=end)
        })
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<String>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines.map(|s| s.to_string()).collect())
    }

    fn part_1(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = values
            .iter()
            .map(|s| s.len() - s.iter_clusters().count())
            .sum::<usize>();
        Ok(value)
    }

    fn part_2(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = values
            .iter()
            .map(|s| format!("{s:?}").len() - s.len())
            .sum::<usize>();
        Ok(value)
    }
}
