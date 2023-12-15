use std::{collections::HashMap, fmt::Display, str::FromStr};

use aoc_utils::prelude::*;

pub struct Record {
    springs: Vec<Option<bool>>,
    groups: Vec<u8>,
}

impl FromStr for Record {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (springs, groups) = s
            .split_ascii_whitespace()
            .collect_tuple()
            .ok_or(Error::InvalidString(s.to_string()))?;

        let springs = springs
            .chars()
            .map(|c| match c {
                '.' => Ok(Some(false)),
                '#' => Ok(Some(true)),
                '?' => Ok(None),
                _ => Err(Error::UnknownChar(c)),
            })
            .collect::<Result<_, _>>()?;

        let groups = groups
            .split(',')
            .map(|val| val.parse())
            .collect::<Result<_, _>>()?;

        Ok(Self { springs, groups })
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.springs.iter().try_for_each(|spring| {
            let c = match spring {
                Some(true) => '#',
                Some(false) => '.',
                None => '?',
            };
            write!(f, "{c}")
        })?;
        write!(f, " ")?;

        self.groups.iter().enumerate().try_for_each(|(i, group)| {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{group}")
        })
    }
}

impl Record {
    fn count_possible(&self) -> usize {
        let mut cache = HashMap::new();
        Self::count_possible_impl(&self.springs, &self.groups, &mut cache)
    }

    fn count_possible_impl(
        springs: &[Option<bool>],
        groups: &[u8],
        cache: &mut HashMap<(Vec<Option<bool>>, Vec<u8>), usize>,
    ) -> usize {
        let key = (
            springs.iter().cloned().collect(),
            groups.iter().cloned().collect(),
        );

        if let Some(&cached) = cache.get(&key) {
            return cached;
        }

        let remaining_group_sum =
            groups.iter().map(|g| *g as usize).sum::<usize>();
        let (minimum_true_values, maximum_true_values) =
            springs
                .iter()
                .fold((0, 0), |(min, max), spring| match spring {
                    Some(true) => (min + 1, max + 1),
                    Some(false) => (min, max),
                    None => (min, max + 1),
                });

        let result = if maximum_true_values < remaining_group_sum
            || remaining_group_sum < minimum_true_values
        {
            0
        } else if let Some(&group) = groups.get(0) {
            let group = group as usize;

            // For all regions that could fit a group of the next size
            (0..=springs.len() - group)
                // So long as we don't skip past a known true value,
                .take_while_inclusive(|&i| match springs[i] {
                    None => true,
                    Some(false) => true,
                    Some(true) => false,
                })
                // all items in the group may be true,
                .filter(|i| {
                    (0..group).all(|offset| match springs[i + offset] {
                        None => true,
                        Some(true) => true,
                        Some(false) => false,
                    })
                })
                // and the item after the group is either the end or may
                // contain a false value,
                .filter(|i| match springs.get(i + group) {
                    None => true,
                    Some(None) => true,
                    Some(Some(true)) => false,
                    Some(Some(false)) => true,
                })
                // then recursively count the number of possible arrangements
                .map(|i| {
                    let earliest_next_group =
                        usize::min(i + group + 1, springs.len());
                    Self::count_possible_impl(
                        &springs[earliest_next_group..],
                        &groups[1..],
                        cache,
                    )
                })
                // and sum over all possible locations
                .sum()
        } else {
            1
        };

        cache.insert(key, result);

        result
    }

    fn unfold(&self) -> Self {
        let springs = [&self.springs; 5]
            .into_iter()
            .with_position()
            .flat_map(|(pos, springs)| {
                let after = match pos {
                    itertools::Position::Last => None,
                    _ => Some(None),
                };
                springs.into_iter().cloned().chain(after)
            })
            .collect();
        let groups = [&self.groups; 5].into_iter().flatten().cloned().collect();
        Self { springs, groups }
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 1;

    type ParsedInput = Vec<Record>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        records: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let arrangements = records
            .iter()
            .map(|record| record.count_possible())
            .sum::<usize>();

        Ok(arrangements)
    }

    fn part_2(
        records: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let arrangements = records
            .iter()
            .map(|record| record.unfold())
            .map(|record| record.count_possible())
            .sum::<usize>();

        Ok(arrangements)
    }
}
