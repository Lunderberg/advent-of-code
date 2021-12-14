#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

use std::collections::HashMap;

pub struct Day14;

#[derive(Debug)]
struct Polymer {
    elements: String,
}

#[derive(Debug)]
struct InsertionRules {
    rules: HashMap<(char, char), char>,
}

impl Polymer {
    fn apply_rules(&self, rules: &InsertionRules) -> Self {
        Self {
            elements: self
                .elements
                .chars()
                .map(|element| Some(element))
                .interleave(
                    self.elements
                        .chars()
                        .tuple_windows()
                        .map(|(a, b)| rules.rules.get(&(a, b)).copied()),
                )
                .flatten()
                .collect(),
        }
    }

    fn minmax_diff(&self) -> Result<usize, Error> {
        self.elements
            .chars()
            .counts()
            .into_iter()
            .map(|(_k, v)| v)
            .minmax()
            .into_option()
            .map(|(min, max)| max - min)
            .ok_or(Error::NoneError)
    }
}

impl Day14 {
    fn parse_inputs(&self) -> Result<(Polymer, InsertionRules), Error> {
        let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        //let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let mut line_iter = puzzle_input.lines();
        let polymer = Polymer {
            elements: line_iter
                .by_ref()
                .take_while(|line| line.len() > 0)
                .map(|line| line.chars().collect())
                .exactly_one()?,
        };

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
            .minmax_diff();

        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        //let puzzle_input = self.puzzle_input(PuzzleInput::User)?;
        let result = ();
        Ok(Box::new(result))
    }
}
