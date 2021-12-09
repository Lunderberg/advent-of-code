#![allow(dead_code)]
#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

pub struct Day08;

#[derive(Debug)]
struct LightSequence {
    unique_patterns: Vec<String>,
    outputs: Vec<String>,
}

impl std::str::FromStr for LightSequence {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.lines()
            .exactly_one()?
            .split('|')
            .tuples()
            .map(|(a, b)| LightSequence {
                unique_patterns: a.split(' ').map(|s| s.to_string()).collect(),
                outputs: b.split(' ').map(|s| s.to_string()).collect(),
            })
            .exactly_one()?)
    }
}

type Segment = u8;

fn active_segments(digit: u8) -> Result<impl Iterator<Item = Segment>, Error> {
    Ok(match digit {
        0 => vec![0, 1, 2, 4, 5, 6],
        1 => vec![2, 5],
        2 => vec![0, 2, 3, 4, 6],
        3 => vec![0, 2, 3, 5, 6],
        4 => vec![1, 2, 3, 5],
        5 => vec![0, 1, 3, 5, 6],
        6 => vec![0, 1, 3, 4, 5, 6],
        7 => vec![0, 2, 5],
        8 => vec![0, 1, 2, 3, 4, 5, 6],
        9 => vec![0, 1, 2, 3, 5, 6],
        _ => Err(Error::InvalidDigit(digit))?,
    }
    .into_iter()
    .map(|i| i as Segment))
}

impl Day08 {
    fn get_light_sequence(&self) -> Result<LightSequence, Error> {
        let puzzle_input = self.puzzle_input(PuzzleInput::Example(1))?;
        // let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        puzzle_input.lines().join("").parse()
    }
}

impl Puzzle for Day08 {
    fn day(&self) -> i32 {
        8
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        //let result = self.get_light_sequence();
        //let result = (0..=9).permutations(10).collect::<Vec<Vec<u8>>>();
        let result = (0..=3)
            .filter_permute(4, |items| {
                println!("Testing {:?}", items);
                items.get(1).map(|&val| val == 2).unwrap_or(true)
            })
            .collect::<Vec<Vec<u8>>>();
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result = ();
        Ok(Box::new(result))
    }
}

pub trait FilterPermutable: Iterator {
    fn filter_permute<V>(self, k: usize, validity: V) -> FilterPermute<Self, V>
    where
        Self: Sized,
        Self::Item: Clone,
        V: FnMut(Vec<Self::Item>) -> bool,
    {
        FilterPermute::new(self, k, validity)
    }
}

impl<T> FilterPermutable for T where T: Iterator {}

pub struct FilterPermute<I, V>
where
    I: Iterator,
    I::Item: Clone,
    V: FnMut(Vec<I::Item>) -> bool,
{
    items: Vec<I::Item>,
    permutation_iter: itertools::structs::Permutations<std::ops::Range<usize>>,
    validity: V,
    most_recent_success: Vec<usize>,
    most_recent_failure: Option<Vec<usize>>,
}

impl<I, V> FilterPermute<I, V>
where
    I: Iterator,
    I::Item: Clone,
    V: FnMut(Vec<I::Item>) -> bool,
{
    fn new(iter: I, k: usize, validity: V) -> Self {
        let items: Vec<_> = iter.collect();
        let permutation_iter = (0..items.len()).permutations(k);
        Self {
            items,
            permutation_iter,
            validity,
            most_recent_success: Vec::new(),
            most_recent_failure: None,
        }
    }
}

impl<I, V> Iterator for FilterPermute<I, V>
where
    I: Iterator,
    I::Item: Clone,
    V: FnMut(Vec<I::Item>) -> bool,
{
    type Item = Vec<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let permutation = self.permutation_iter.next()?;

            let is_subset_of_failure = self
                .most_recent_failure
                .as_ref()
                .map_or(false, |failed_indices| {
                    failed_indices
                        .iter()
                        .enumerate()
                        .all(|(pos, index)| *index == permutation[pos])
                });
            if is_subset_of_failure {
                continue;
            }
            // Once we've moved past subsets of the most recent
            // failure, we never revisit that region, so we don't need
            // to check for it until the next failure.
            self.most_recent_failure = None;

            let first_test_required = self
                .most_recent_success
                .iter()
                .enumerate()
                .filter(|&(pos, index)| *index == permutation[pos])
                .map(|(pos, _index)| pos + 1)
                .next()
                .unwrap_or(0);

            let first_failing_pos = (first_test_required..permutation.len())
                .filter(|&pos| {
                    let items = (0..=pos)
                        .map(|i| self.items[permutation[i]].clone())
                        .collect();
                    !(self.validity)(items)
                })
                .next();

            // If one of the tests failed, mark the location of the
            // most recent success/failure, then move on.
            if let Some(pos) = first_failing_pos {
                self.most_recent_failure = Some(permutation[0..=pos].to_vec());
                self.most_recent_success = permutation[0..pos].to_vec();
                continue;
            }

            // All checks have passed, so we can return this.
            return Some(
                permutation
                    .iter()
                    .map(|&index| self.items[index].clone())
                    .collect(),
            );
        }
    }
}
