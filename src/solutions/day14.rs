#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::Puzzle;

use itertools::Itertools;

use std::collections::HashMap;

pub struct Day14;

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
                vec![(*a, *counts), (*b, *counts)].into_iter()
            })
            .chain(vec![(self.first, 1), (self.last, 1)].into_iter())
            .into_group_map()
            .into_iter()
            .map(|(_element, counts)| counts.iter().sum::<usize>() / 2)
            .minmax()
            .into_option()
            .map(|(min, max)| max - min)
            .ok_or(Error::NoneError)
    }
}

impl Puzzle for Day14 {
    const DAY: u8 = 14;
    const IMPLEMENTED: bool = true;
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = (Polymer, InsertionRules);
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let polymer = lines
            .by_ref()
            .take_while(|line| line.len() > 0)
            .map(|line| line.parse::<Polymer>())
            .exactly_one()??;

        let rules = InsertionRules {
            rules: lines
                .map(|line| -> Result<_, Error> {
                    Ok(line
                        .split(" -> ")
                        .tuples()
                        .map(|(before, after)| -> Result<_, Error> {
                            let initial = before
                                .chars()
                                .tuples::<(_, _)>()
                                .exactly_one()?;
                            let insertion = after.chars().exactly_one()?;
                            Ok((initial, insertion))
                        })
                        .exactly_one()??)
                })
                .try_collect()?,
        };

        Ok((polymer, rules))
    }

    type Part1Result = usize;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        let (polymer, rules) = parsed;

        (0..10)
            .fold(polymer.clone(), |state: Polymer, _| {
                state.apply_rules(&rules)
            })
            .minmax_diff()
    }

    type Part2Result = usize;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        let (polymer, rules) = parsed;

        (0..40)
            .fold(polymer.clone(), |state, _| state.apply_rules(&rules))
            .minmax_diff()
    }
}
