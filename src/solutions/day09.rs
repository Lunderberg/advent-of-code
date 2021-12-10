#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

use std::collections::HashSet;

pub struct Day09;

#[derive(Debug)]
struct HeightMap {
    x_size: usize,
    y_size: usize,
    values: Vec<u8>,
}

impl HeightMap {
    fn index_of(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.x_size && y < self.y_size {
            Some(y * self.x_size + x)
        } else {
            None
        }
    }

    fn iter(&self) -> impl Iterator<Item = (usize, usize, u8)> + '_ {
        let x_size = self.x_size;
        self.values
            .iter()
            .enumerate()
            .map(move |(i, val)| (i % x_size, i / x_size, *val))
    }

    fn get(&self, x: usize, y: usize) -> Option<u8> {
        self.index_of(x, y).map(|i| self.values[i])
    }

    fn adjacent_points(
        &self,
        x: usize,
        y: usize,
    ) -> impl Iterator<Item = (usize, usize)> + '_ {
        [(1, 0), (0, 1), (-1, 0), (0, -1)]
            .iter()
            .map(move |(dx, dy)| {
                let x_adjacent = (x as i64) + dx;
                let y_adjacent = (y as i64) + dy;
                if x_adjacent >= 0
                    && y_adjacent >= 0
                    && x_adjacent < (self.x_size as i64)
                    && y_adjacent < (self.y_size as i64)
                {
                    Some((x_adjacent as usize, y_adjacent as usize))
                } else {
                    None
                }
            })
            .flatten()
    }

    fn low_points(&self) -> impl Iterator<Item = (usize, usize, u8)> + '_ {
        self.iter().filter(move |&(x, y, height)| {
            self.adjacent_points(x, y).all(|(x_other, y_other)| {
                self.get(x_other, y_other)
                    .map_or(true, |height_adjacent| height_adjacent > height)
            })
        })
    }

    fn basin_points(&self, low_point: (usize, usize)) -> Vec<(usize, usize)> {
        let mut search_stack: Vec<(usize, usize)> = vec![low_point];

        let mut touched: HashSet<(usize, usize)> = HashSet::new();
        touched.insert(low_point);

        let mut output: Vec<(usize, usize)> = Vec::new();

        while search_stack.len() > 0 {
            let point = search_stack.pop().unwrap();
            if self.get(point.0, point.1).unwrap() != 9 {
                output.push(point);
                self.adjacent_points(point.0, point.1)
                    .filter(|p| !touched.contains(p))
                    .collect::<Vec<_>>()
                    .iter()
                    .for_each(|p| {
                        search_stack.push(*p);
                        touched.insert(*p);
                    });
            }
        }
        output
    }
}

impl std::str::FromStr for HeightMap {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (line_num, line_length, values): (Vec<_>, Vec<_>, Vec<_>) = s
            .lines()
            .enumerate()
            .map(|(line_num, line)| {
                line.chars().map(move |c| -> Result<_, Error> {
                    let value = c.to_string().parse::<u8>()?;
                    Ok((line_num, line.len(), value))
                })
            })
            .flatten()
            .collect::<Result<Vec<(usize, usize, u8)>, _>>()?
            .into_iter()
            .multiunzip();

        let x_size = line_length.into_iter().unique().exactly_one()?;
        let y_size = line_num.into_iter().max().ok_or(Error::NoneError)? + 1;

        Ok(HeightMap {
            x_size,
            y_size,
            values,
        })
    }
}

impl std::ops::Index<(usize, usize)> for HeightMap {
    type Output = u8;
    fn index(&self, (x, y): (usize, usize)) -> &u8 {
        &self.values[self.index_of(x, y).unwrap()]
    }
}

impl Day09 {
    fn parse_height_map(&self) -> Result<HeightMap, Error> {
        // let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;
        puzzle_input.parse()
    }
}

impl Puzzle for Day09 {
    fn day(&self) -> i32 {
        9
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result = self
            .parse_height_map()?
            .low_points()
            .map(|(_x, _y, height)| (height + 1) as u64)
            .sum::<u64>();
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let height_map = self.parse_height_map()?;

        let result = height_map
            .low_points()
            .map(|(x, y, _height)| height_map.basin_points((x, y)).len())
            .sorted_by_key(|&i| -(i as i64))
            .take(3)
            .product::<usize>();
        Ok(Box::new(result))
    }
}
