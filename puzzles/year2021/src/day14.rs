use aoc_utils::prelude::*;

use std::collections::HashMap;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

#[derive(Debug, Clone)]
pub struct Polymer {
    pair_counts: HashMap<(char, char), usize>,
    first: char,
    last: char,
}

#[derive(Debug)]
pub struct InsertionRules {
    rules: HashMap<(char, char), char>,
}

impl std::str::FromStr for Polymer {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let pair_counts = s.chars().tuple_windows::<(_, _)>().counts();
        let first = s.chars().next().ok_or(Error::NoneError)?;
        let last = s.chars().last().ok_or(Error::NoneError)?;
        Ok(Self {
            first,
            last,
            pair_counts,
        })
    }
}

impl Polymer {
    fn apply_rules(&self, rules: &InsertionRules) -> Self {
        let pair_counts = self
            .pair_counts
            .iter()
            .map(|(pair, counts)| (*pair, *counts))
            .flat_map(|((a, b), counts)| {
                rules
                    .rules
                    .get(&(a, b))
                    .copied()
                    .map(|insert| vec![(a, insert), (insert, b)].into_iter())
                    .or_else(|| Some(vec![(a, b)].into_iter()))
                    .unwrap()
                    .map(move |pair| (pair, counts))
            })
            .into_group_map()
            .into_iter()
            .map(|(pair, count_vec)| (pair, count_vec.iter().sum::<usize>()))
            .collect();
        Self {
            first: self.first,
            last: self.last,
            pair_counts,
        }
    }

    fn minmax_diff(&self) -> Result<usize, Error> {
        self.pair_counts
            .iter()
            .flat_map(|((a, b), counts)| {
                [(*a, *counts), (*b, *counts)].into_iter()
            })
            .chain([(self.first, 1), (self.last, 1)])
            .into_group_map()
            .into_values()
            .map(|counts| counts.iter().sum::<usize>() / 2)
            .minmax()
            .into_option()
            .map(|(min, max)| max - min)
            .ok_or(Error::NoneError)
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = (Polymer, InsertionRules);
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let polymer = lines
            .by_ref()
            .take_while(|line| !line.is_empty())
            .map(|line| line.parse::<Polymer>())
            .exactly_one_or_err()??;

        let rules = InsertionRules {
            rules: lines
                .map(|line| -> Result<_, Error> {
                    line.split(" -> ")
                        .tuples()
                        .map(|(before, after)| -> Result<_, Error> {
                            let initial = before
                                .chars()
                                .tuples::<(_, _)>()
                                .exactly_one_or_err()?;
                            let insertion =
                                after.chars().exactly_one_or_err()?;
                            Ok((initial, insertion))
                        })
                        .exactly_one_or_err()?
                })
                .try_collect()?,
        };

        Ok((polymer, rules))
    }

    fn part_1(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let (polymer, rules) = parsed;

        (0..10)
            .fold(polymer.clone(), |state: Polymer, _| {
                state.apply_rules(rules)
            })
            .minmax_diff()
    }

    fn part_2(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let (polymer, rules) = parsed;

        (0..40)
            .fold(polymer.clone(), |state, _| state.apply_rules(rules))
            .minmax_diff()
    }
}
