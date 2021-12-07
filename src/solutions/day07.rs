#![allow(unused_imports)]
use utils::Error;
use utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

pub struct Day07;

impl Day07 {
    fn parse_crabs(&self) -> Result<Vec<i64>, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        Ok(puzzle_input
            .lines()
            .next()
            .unwrap()
            .split(',')
            .map(|s| s.parse::<i64>())
            .collect::<Result<Vec<_>, _>>()?)
    }
}

impl Puzzle for Day07 {
    fn day(&self) -> i32 {
        7
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let crab_pos = self.parse_crabs()?;

        let (low, high) = match crab_pos.iter().map(|x| *x).minmax() {
            itertools::MinMaxResult::NoElements => Err(Error::CannotFindMinMax),
            itertools::MinMaxResult::OneElement(_) => {
                Err(Error::CannotFindMinMax)
            }
            itertools::MinMaxResult::MinMax(low, high) => Ok((low, high)),
        }?;

        let lowest_fuel = (low..=high)
            .map(|pos| crab_pos.iter().map(|c| (pos - c).abs()).sum::<i64>())
            .min();

        let result = lowest_fuel;
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let crab_pos = self.parse_crabs()?;

        let (low, high) = match crab_pos.iter().map(|x| *x).minmax() {
            itertools::MinMaxResult::NoElements => Err(Error::CannotFindMinMax),
            itertools::MinMaxResult::OneElement(_) => {
                Err(Error::CannotFindMinMax)
            }
            itertools::MinMaxResult::MinMax(low, high) => Ok((low, high)),
        }?;

        let lowest_fuel = (low..=high)
            .map(|pos| {
                crab_pos
                    .iter()
                    .map(|c| {
                        let steps = (c - pos).abs();
                        steps * (steps + 1) / 2
                    })
                    .sum::<i64>()
            })
            .min();

        let result = lowest_fuel;
        Ok(Box::new(result))
    }
}
