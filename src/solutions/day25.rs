#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::GridMap;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops;
use std::str::FromStr;

use itertools::Itertools;

pub struct Day25;

#[derive(Debug, PartialEq, Eq)]
struct CucumberMap {
    cucumbers: HashMap<Vector2, Cucumber>,
    map_size: Vector2,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
struct Vector2([i64; 2]);

impl ops::Add for Vector2 {
    type Output = Vector2;
    fn add(self, rhs: Self) -> Self {
        let mut values = [0; 2];
        self.0
            .iter()
            .zip(rhs.0.iter())
            .map(|(a, b)| a + b)
            .zip(values.iter_mut())
            .for_each(|(val, out)| {
                *out = val;
            });
        Self(values)
    }
}

impl ops::Rem for Vector2 {
    type Output = Vector2;
    fn rem(self, rhs: Self) -> Self {
        let mut values = [0; 2];
        self.0
            .iter()
            .zip(rhs.0.iter())
            .map(|(a, b)| a % b)
            .zip(values.iter_mut())
            .for_each(|(val, out)| {
                *out = val;
            });
        Self(values)
    }
}

#[derive(Debug)]
struct Tile {
    contents: Option<Cucumber>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cucumber {
    South,
    East,
}

impl Display for CucumberMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let grid_map: GridMap<Tile> = (0..self.map_size.0[0])
            .cartesian_product(0..self.map_size.0[1])
            .map(|(i, j)| {
                let pos = Vector2([i, j]);
                let tile = Tile {
                    contents: self.cucumbers.get(&pos).copied(),
                };
                (i as usize, j as usize, tile)
            })
            .collect();
        write!(f, "{}", grid_map)
    }
}

impl CucumberMap {
    fn after_advance(&self) -> Self {
        self.after_advance_herd(Cucumber::East)
            .after_advance_herd(Cucumber::South)
    }

    fn after_advance_herd(&self, moving_herd: Cucumber) -> Self {
        let cucumbers = self
            .cucumbers
            .iter()
            .map(|(&pos, &cuke)| {
                let new_pos = if cuke == moving_herd {
                    let target_pos = (pos + cuke.step_delta()) % self.map_size;
                    if self.cucumbers.contains_key(&target_pos) {
                        pos
                    } else {
                        target_pos
                    }
                } else {
                    pos
                };
                (new_pos, cuke)
            })
            .collect();
        Self {
            cucumbers,
            map_size: self.map_size,
        }
    }
}

impl Cucumber {
    fn step_delta(&self) -> Vector2 {
        match self {
            Cucumber::South => Vector2([0, 1]),
            Cucumber::East => Vector2([1, 0]),
        }
    }
}

impl FromStr for Tile {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let contents = match s {
            ">" => Some(Cucumber::East),
            "v" => Some(Cucumber::South),
            "." => None,
            _ => Err(Error::InvalidString(s.to_string()))?,
        };
        Ok(Self { contents })
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let c = match self.contents {
            Some(Cucumber::East) => '>',
            Some(Cucumber::South) => 'v',
            None => '.',
        };

        write!(f, "{}", c)
    }
}

impl Day25 {
    fn parse_cucumbers(&self) -> Result<CucumberMap, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let map = puzzle_input.lines().collect::<GridMap<Tile>>();
        let map_size = Vector2([map.x_size as i64, map.y_size as i64]);
        let cucumbers = map
            .iter()
            .filter_map(|(grid_pos, tile)| {
                tile.contents.as_ref().map(|cuke| {
                    let (i, j) = grid_pos.as_xy(&map);
                    let pos = Vector2([i, j]);
                    (pos, *cuke)
                })
            })
            .collect();

        Ok(CucumberMap {
            cucumbers,
            map_size,
        })
    }
}

impl Puzzle for Day25 {
    fn day(&self) -> i32 {
        25
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result =
            std::iter::successors(Some(self.parse_cucumbers()?), |map| {
                Some(map.after_advance())
            })
            .enumerate()
            .tuples()
            .find_map(|((_, before), (i, after))| (before == after).then(|| i))
            .unwrap();
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        //let puzzle_input = self.puzzle_input(PuzzleInput::User)?;
        let result = ();
        Ok(Box::new(result))
    }
}
