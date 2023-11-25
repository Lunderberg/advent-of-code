#![allow(unused_imports)]
use crate::utils::geometry::Vector;
use crate::utils::GridMap;
use crate::{Error, Puzzle};

use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct RockPath {
    points: Vec<Vector<2>>,
}

struct SandSimulation {
    lowest_rock: i64,
    rock: HashSet<Vector<2>>,
    sand: HashSet<Vector<2>>,
    sand_source: Vector<2>,
    has_floor: bool,
}

impl std::str::FromStr for RockPath {
    type Err = Error;
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let points = line
            .split(" -> ")
            .map(|pair| -> Result<_, Error> {
                let (a, b) = pair
                    .split(',')
                    .map(|val| val.parse::<i64>())
                    .tuples()
                    .exactly_one()?;
                Ok(Vector::new([a?, b?]))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RockPath { points })
    }
}

impl RockPath {
    /// All points covered by this path
    fn iter_path(&self) -> impl Iterator<Item = Vector<2>> + '_ {
        self.points
            .iter()
            .tuple_windows()
            .flat_map(|(a, b)| a.cardinal_points_to(b))
    }
}

enum SimulatedSand {
    Stopped(Vector<2>),
    InMotion(Vector<2>),
    FallingForever,
}

impl SandSimulation {
    fn new(
        rock_paths: &[RockPath],
        sand_source: Vector<2>,
        has_floor: bool,
    ) -> Self {
        let rock: HashSet<_> = rock_paths
            .iter()
            .flat_map(|path| path.iter_path())
            .collect();
        Self {
            lowest_rock: rock.iter().map(|loc| loc.y()).max().unwrap(),
            rock,
            sand: HashSet::new(),
            sand_source,
            has_floor,
        }
    }

    fn fill(&mut self) {
        while let Some(pos) = self.next_sand_pos() {
            self.sand.insert(pos);
        }
    }

    fn next_sand_pos(&self) -> Option<Vector<2>> {
        self.sand_path()
            .take_while(|sand| !matches!(sand, SimulatedSand::FallingForever))
            .find_map(|sand| {
                if let SimulatedSand::Stopped(loc) = sand {
                    Some(loc)
                } else {
                    None
                }
            })
    }

    fn sand_path(&self) -> impl Iterator<Item = SimulatedSand> + '_ {
        let initial = (!self.sand.contains(&self.sand_source))
            .then_some(SimulatedSand::InMotion(self.sand_source));
        std::iter::successors(initial, move |prev| match prev {
            SimulatedSand::Stopped(_) => None,
            SimulatedSand::InMotion(loc) => {
                if !self.has_floor && loc.y() > self.lowest_rock {
                    Some(SimulatedSand::FallingForever)
                } else {
                    let downward = *loc + [0, 1].into();
                    let left = *loc + [-1, 1].into();
                    let right = *loc + [1, 1].into();
                    Some(
                        [downward, left, right]
                            .iter()
                            .find(|loc| {
                                !self.sand.contains(loc)
                                    && !self.rock.contains(loc)
                                    && (!self.has_floor
                                        || loc.y() < self.lowest_rock + 2)
                            })
                            .map(|new_loc| SimulatedSand::InMotion(*new_loc))
                            .unwrap_or_else(|| SimulatedSand::Stopped(*loc)),
                    )
                }
            }
            SimulatedSand::FallingForever => None,
        })
    }
}

impl Display for SandSimulation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let next_path: HashSet<_> = self
            .sand_path()
            .take_while(|sand| !matches!(sand, SimulatedSand::FallingForever))
            .filter_map(|sand| match sand {
                SimulatedSand::Stopped(loc) => Some(loc),
                SimulatedSand::InMotion(loc) => Some(loc),
                SimulatedSand::FallingForever => None,
            })
            .collect();

        let (min, max) = std::iter::empty()
            .chain(self.rock.iter())
            .chain(self.sand.iter())
            .chain(next_path.iter())
            .chain(std::iter::once(&self.sand_source))
            .fold(None, |acc: Option<(Vector<2>, Vector<2>)>, point| {
                if let Some((min, max)) = acc {
                    Some((
                        min.zip_map(*point, |&a, &b| a.min(b)),
                        max.zip_map(*point, |&a, &b| a.max(b)),
                    ))
                } else {
                    Some((*point, *point))
                }
            })
            .unwrap();
        let display_min = min - [2, 2].into();
        let display_max = max + [2, 2].into();
        let line_length = (display_max.x() - display_min.x() + 1) as usize;
        let num_lines = (display_max.y() - display_min.y() + 1) as usize;
        let num_points = line_length * num_lines;

        let map: GridMap<char> = (0..num_points)
            .map(|i| (i.rem_euclid(line_length), i.div_euclid(line_length)))
            .map(|(x, y)| {
                let point: Vector<2> = [
                    display_min.x() + (x as i64),
                    display_min.y() + (y as i64),
                ]
                .into();
                let c = if self.rock.contains(&point) {
                    '#'
                } else if point == self.sand_source {
                    '+'
                } else if self.sand.contains(&point) {
                    'o'
                } else if next_path.contains(&point) {
                    '~'
                } else if self.has_floor && (y as i64) >= self.lowest_rock + 4 {
                    '#'
                } else {
                    '.'
                };
                (x, y, c)
            })
            .collect();
        write!(f, "{map}")
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<RockPath>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        paths: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let mut sim = SandSimulation::new(paths, [500, 0].into(), false);
        // println!("Before:\n{sim}");
        sim.fill();
        // println!("After:\n{sim}");
        Ok(sim.sand.len())
    }

    fn part_2(
        paths: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let mut sim = SandSimulation::new(paths, [500, 0].into(), true);
        //println!("Before:\n{sim}");
        sim.fill();
        //println!("After:\n{sim}");
        Ok(sim.sand.len())
    }
}
