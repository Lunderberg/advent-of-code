#![allow(unused_imports)]
use crate::utils::{Adjacency, GridMap, GridPos};
use crate::{Error, Puzzle};

use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

pub struct Day11;

#[derive(Debug, Clone)]
pub struct OctopusMap {
    total_flashes: usize,
    map: GridMap<Octopus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Octopus {
    Charging(u8),
    Flashing,
    Flashed,
}

impl Display for Octopus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Octopus::*;
        match self {
            Charging(val) => write!(f, "{}", val),
            Flashing => write!(f, "F"),
            Flashed => write!(f, "."),
        }
    }
}

impl Octopus {
    fn accumulate(&mut self) {
        *self = self.after_accumulate();
    }

    fn after_accumulate(&self) -> Octopus {
        use Octopus::*;
        match self {
            Charging(val) => {
                if *val == 9 {
                    Flashing
                } else {
                    Charging(val + 1)
                }
            }
            _ => *self,
        }
    }

    fn ready_to_flash(&self) -> bool {
        *self != self.after_try_flash()
    }

    fn try_flash(&mut self) {
        *self = self.after_try_flash();
    }

    fn after_try_flash(&self) -> Octopus {
        use Octopus::*;
        match self {
            Flashing => Flashed,
            _ => *self,
        }
    }

    fn reset(&mut self) {
        *self = self.after_reset();
    }

    fn after_reset(&self) -> Octopus {
        use Octopus::*;
        match self {
            Flashed => Charging(0),
            _ => *self,
        }
    }
}

impl OctopusMap {
    fn adjacent_points(
        &self,
        pos: GridPos,
    ) -> impl Iterator<Item = GridPos> + '_ {
        self.map.adjacent_points(pos, Adjacency::Queen)
    }

    #[allow(dead_code)]
    fn iterate_stack_based(&mut self) {
        self.map
            .iter_mut()
            .for_each(|(_pos, octo)| octo.accumulate());

        let mut flash_stack: Vec<_> = self
            .map
            .iter()
            .flat_map(|(pos, octo)| octo.ready_to_flash().then(|| pos))
            .collect();
        let mut all_flashes: HashSet<_> = flash_stack.iter().copied().collect();

        while flash_stack.len() > 0 {
            let flashing = flash_stack.pop().unwrap().into();
            self.map[flashing].try_flash();

            self.adjacent_points(flashing)
                .filter(|adj| !all_flashes.contains(&adj))
                .collect::<Vec<_>>()
                .into_iter()
                .filter_map(|adj| {
                    self.map[adj].accumulate();
                    self.map[adj].ready_to_flash().then(|| adj)
                })
                .collect::<Vec<_>>()
                .into_iter()
                .for_each(|adj| {
                    flash_stack.push(adj);
                    all_flashes.insert(adj);
                });
        }
        self.total_flashes += all_flashes.len();

        self.map.iter_mut().for_each(|(_pos, octo)| octo.reset());
    }

    #[allow(dead_code)]
    fn iterate_loop_all(&mut self) {
        self.map
            .iter_mut()
            .for_each(|(_pos, octo)| octo.accumulate());

        loop {
            let flashing: Vec<_> = self
                .map
                .iter_mut()
                .filter_map(|(pos, octo)| {
                    let orig = *octo;
                    octo.try_flash();
                    (orig != *octo).then(|| pos)
                })
                .collect();

            if flashing.len() == 0 {
                break;
            }

            self.total_flashes += flashing.len();

            flashing
                .into_iter()
                .flat_map(|i| self.adjacent_points(i))
                .collect::<Vec<_>>()
                .into_iter()
                .for_each(|pos| self.map[pos].accumulate());
        }

        self.map.iter_mut().for_each(|(_pos, octo)| octo.reset());
    }

    fn iterate(&mut self) {
        // TODO: Benchmark these two.
        self.iterate_stack_based();
        // self.iterate_loop_all();
    }

    fn is_synchronized(&self) -> bool {
        self.map.iter().map(|(_pos, octo)| octo).all_equal()
    }
}

impl std::str::FromStr for Octopus {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Octopus::*;
        let char = s.chars().exactly_one()?;
        match char {
            '0'..='9' => Ok(Charging(s.parse::<u8>()?)),
            'F' => Ok(Flashing),
            '.' => Ok(Flashed),
            _ => Err(Error::UnknownChar(char)),
        }
    }
}

impl Puzzle for Day11 {
    const DAY: u8 = 11;
    const IMPLEMENTED: bool = true;
    const EXAMPLE_NUM: u8 = 1;

    type ParsedInput = OctopusMap;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(OctopusMap {
            total_flashes: 0,
            map: lines.collect(),
        })
    }

    type Part1Result = usize;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        let mut map = parsed.clone();

        (0..100).for_each(|_i| map.iterate());
        Ok(map.total_flashes)
    }

    type Part2Result = usize;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        Ok(std::iter::repeat(())
            .scan(parsed.clone(), |map, _| {
                map.iterate();
                Some(map.is_synchronized())
            })
            .enumerate()
            .flat_map(|(iter, b)| b.then(|| iter + 1))
            .next()
            .unwrap())
    }
}
