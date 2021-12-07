#![allow(unused_imports)]
use std::collections::VecDeque;

use itertools::Itertools;

use utils::Error;
use utils::{Puzzle, PuzzleExtensions, PuzzleInput};

pub struct Day06;

impl Day06 {
    fn parse_fish(&self) -> Result<VecDeque<u64>, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        // Use a mutable vector to accumulate the results.
        let mut output: VecDeque<u64> = (0..9).map(|_i| 0).collect();
        puzzle_input
            .lines()
            .next()
            .unwrap()
            .split(',')
            .try_for_each(|s| -> Result<(), Error> {
                Ok(output[s.parse::<usize>()?] += 1)
            })?;

        // // HashSet of counts, then convert to vector.
        // let counts = puzzle_input
        //     .split(',')
        //     .map(|s| s.parse::<usize>())
        //     .collect::<Result<Vec<_>, _>>()?
        //     .into_iter()
        //     .counts();
        // let output = (0..9).map(|i| counts[&i]).collect();

        // // Chained iterators, but reconstruct the VecDeque for each
        // // iteration, unless the optimizer can do something about it.
        // let output = puzzle_input
        //     .split(',')
        //     .map(|s| s.parse::<usize>())
        //     .try_fold(
        //         (0..9).map(|_i| 0).collect::<VecDeque<_>>(),
        //         |acc, res| {
        //             res.and_then(|val| {
        //                 Ok(acc
        //                     .iter()
        //                     .enumerate()
        //                     .map(|(i, num)| num + ((i == val) as u64))
        //                     .collect())
        //             })
        //         },
        //     )?;

        Ok(output)
    }

    fn advance_fish_population(num_days: u32, population: &mut VecDeque<u64>) {
        (0..num_days).for_each(|_day| {
            population[7] += population[0];
            population.rotate_left(1);
        });
    }
}

impl Puzzle for Day06 {
    fn day(&self) -> i32 {
        6
    }

    fn implemented(&self) -> bool {
        true
    }

    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let mut population = self.parse_fish()?;
        Self::advance_fish_population(80, &mut population);
        let result = population.iter().sum::<u64>();
        Ok(Box::new(result))
    }

    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let mut population = self.parse_fish()?;
        Self::advance_fish_population(256, &mut population);
        let result = population.iter().sum::<u64>();
        Ok(Box::new(result))
    }
}
