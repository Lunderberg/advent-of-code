#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

use std::collections::HashMap;

pub struct Day14;

#[derive(Debug)]
struct Polymer {
    pair_counts: HashMap<(char, char), usize>,
    first: char,
    last: char,
}

#[derive(Debug)]
struct InsertionRules {
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

impl Day14 {
    fn parse_inputs(&self) -> Result<(Polymer, InsertionRules), Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let mut line_iter = puzzle_input.lines();
        let polymer = line_iter
            .by_ref()
            .take_while(|line| line.len() > 0)
            .map(|line| line.parse::<Polymer>())
            .exactly_one()??;

        let rules = InsertionRules {
            rules: line_iter
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
}

impl Puzzle for Day14 {
    fn day(&self) -> i32 {
        14
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let (polymer, rules) = self.parse_inputs()?;

        let result = (0..10)
            .fold(polymer, |state, _| state.apply_rules(&rules))
            .minmax_diff()?;

        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let (polymer, rules) = self.parse_inputs()?;

        let result = (0..40)
            .fold(polymer, |state, _| state.apply_rules(&rules))
            .minmax_diff()?;

        Ok(Box::new(result))
    }
}
