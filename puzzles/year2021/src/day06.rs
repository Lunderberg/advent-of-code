use aoc_utils::prelude::*;

use std::collections::VecDeque;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl ThisDay {
    fn advance_fish_population(num_days: u32, population: &mut VecDeque<u64>) {
        (0..num_days).for_each(|_day| {
            population[7] += population[0];
            population.rotate_left(1);
        });
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = VecDeque<u64>;
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        // Use a mutable vector to accumulate the results.
        let mut output: VecDeque<u64> = (0..9).map(|_i| 0).collect();
        lines.next().unwrap().split(',').try_for_each(
            |s| -> Result<(), Error> {
                output[s.parse::<usize>()?] += 1;
                Ok(())
            },
        )?;

        // // HashSet of counts, then convert to vector.
        // let counts = lines
        //     .next()
        //     .unwrap()
        //     .split(',')
        //     .map(|s| s.parse::<usize>())
        //     .collect::<Result<Vec<_>, _>>()?
        //     .into_iter()
        //     .counts();
        // let output = (0..9).map(|i| counts[&i] as u64).collect();

        // // Chained iterators, but reconstruct the VecDeque for each
        // // iteration, unless the optimizer can do something about it.
        // let output = lines
        //     .next()
        //     .unwrap()
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

    fn part_1(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let mut population = parsed.clone();
        Self::advance_fish_population(80, &mut population);
        Ok(population.iter().sum::<u64>())
    }

    fn part_2(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let mut population = parsed.clone();
        Self::advance_fish_population(256, &mut population);
        Ok(population.iter().sum::<u64>())
    }
}
