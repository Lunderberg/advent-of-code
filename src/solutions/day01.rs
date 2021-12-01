#![allow(unused_imports)]
use utils::Error;
use utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

pub struct Day01;

impl Puzzle for Day01 {
    fn day(&self) -> i32 {
        1
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let result = puzzle_input
            .lines()
            .map(|line| line.parse::<i32>().map_err(|err| err.into()))
            .collect::<Result<Vec<_>, Error>>()?
            .iter()
            .tuple_windows()
            .filter(|(a, b)| a < b)
            .count();
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let result = puzzle_input
            .lines()
            .map(|line| line.parse::<i32>().map_err(|err| err.into()))
            .collect::<Result<Vec<_>, Error>>()?
            .iter()
            .tuple_windows()
            .map(|(a, b, c)| a + b + c)
            .tuple_windows()
            .filter(|(a, b)| a < b)
            .count();

        Ok(Box::new(result))
    }
}
