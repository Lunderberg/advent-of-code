#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

pub struct Day11;

#[derive(Debug, Clone)]
struct OctopusMap {
    total_flashes: usize,
    x_size: usize,
    y_size: usize,
    values: Vec<Octopus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Octopus {
    Charging(u8),
    Flashing,
    Flashed,
}

impl Display for OctopusMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.values
            .iter()
            .chunks(self.x_size)
            .into_iter()
            .try_for_each(|mut chunk| -> Result<_, std::fmt::Error> {
                chunk.try_for_each(|val| write!(f, "{}", val))?;
                write!(f, "\n")?;
                Ok(())
            })?;
        Ok(())
    }
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
    fn adjacent_points(&self, i: usize) -> impl Iterator<Item = usize> {
        let x0 = (i % self.x_size) as i64;
        let y0 = (i / self.x_size) as i64;
        let x_size = self.x_size as i64;
        let y_size = self.x_size as i64;
        (-1..=1)
            .cartesian_product(-1..=1)
            .filter(|(dx, dy)| *dx != 0 || *dy != 0)
            .map(move |(dx, dy)| {
                let y = y0 + dy;
                let x = x0 + dx;
                if x >= 0 && y >= 0 && x < x_size && y < y_size {
                    Some((y * x_size + x) as usize)
                } else {
                    None
                }
            })
            .flatten()
    }

    #[allow(dead_code)]
    fn iterate_stack_based(&mut self) {
        self.values.iter_mut().for_each(|octo| octo.accumulate());

        let mut flash_stack: Vec<usize> = self
            .values
            .iter()
            .copied()
            .enumerate()
            .flat_map(|(i, octo)| octo.ready_to_flash().then(|| i))
            .collect();
        let mut all_flashes: HashSet<usize> =
            flash_stack.iter().copied().collect();

        while flash_stack.len() > 0 {
            let i = flash_stack.pop().unwrap();
            self.values[i].try_flash();

            self.adjacent_points(i)
                .filter(|i| !all_flashes.contains(i))
                .filter(|i| {
                    self.values[*i].accumulate();
                    self.values[*i].ready_to_flash()
                })
                .collect::<Vec<_>>()
                .into_iter()
                .for_each(|i| {
                    flash_stack.push(i);
                    all_flashes.insert(i);
                });
        }
        self.total_flashes += all_flashes.len();

        self.values.iter_mut().for_each(|octo| octo.reset());
    }

    #[allow(dead_code)]
    fn iterate_loop_all(&mut self) {
        self.values.iter_mut().for_each(|octo| octo.accumulate());

        loop {
            let flashing: Vec<usize> = self
                .values
                .iter_mut()
                .enumerate()
                .filter_map(|(i, octo)| {
                    let orig = *octo;
                    octo.try_flash();
                    (orig != *octo).then(|| i)
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
                .for_each(|i| self.values[i].accumulate());
        }

        self.values.iter_mut().for_each(|octo| octo.reset());
    }

    fn iterate(&mut self) {
        // TODO: Benchmark these two.
        self.iterate_stack_based();
        // self.iterate_loop_all();
    }

    fn is_synchronized(&self) -> bool {
        self.values.iter().all_equal()
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

impl std::str::FromStr for OctopusMap {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (line_num, line_length, values): (Vec<_>, Vec<_>, Vec<_>) = s
            .lines()
            .enumerate()
            .map(|(line_num, line)| {
                line.chars().map(move |c| -> Result<_, Error> {
                    let value = c.to_string().parse::<Octopus>()?;
                    Ok((line_num, line.len(), value))
                })
            })
            .flatten()
            .collect::<Result<Vec<(usize, usize, Octopus)>, _>>()?
            .into_iter()
            .multiunzip();

        let x_size = line_length.into_iter().unique().exactly_one()?;
        let y_size = line_num.into_iter().max().ok_or(Error::NoneError)? + 1;

        Ok(OctopusMap {
            total_flashes: 0,
            x_size,
            y_size,
            values,
        })
    }
}

impl Day11 {
    fn parse_octopodes(&self) -> Result<OctopusMap, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;
        puzzle_input.parse()
    }
}

impl Puzzle for Day11 {
    fn day(&self) -> i32 {
        11
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let mut map = self.parse_octopodes()?;
        (0..100).for_each(|_i| map.iterate());
        let result = map.total_flashes;
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let map = self.parse_octopodes()?;
        let result = std::iter::repeat(())
            .scan(map, |map, _| {
                map.iterate();
                Some(map.is_synchronized())
            })
            .enumerate()
            .flat_map(|(iter, b)| b.then(|| iter + 1))
            .next();
        Ok(Box::new(result))
    }
}
