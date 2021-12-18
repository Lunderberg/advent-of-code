#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Adjacency, GridMap, GridPos};
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

use std::collections::HashSet;

pub struct Day09;

#[derive(Debug)]
struct HeightMap {
    map: GridMap<u8>,
}

impl HeightMap {
    fn adjacent_points(
        &self,
        pos: GridPos,
    ) -> impl Iterator<Item = GridPos> + '_ {
        self.map.adjacent_points(pos, Adjacency::Rook)
    }

    fn low_points(&self) -> impl Iterator<Item = (GridPos, u8)> + '_ {
        self.map.iter().map(|(pos, height)| (pos, *height)).filter(
            move |&(pos, height)| {
                self.adjacent_points(pos).all(|adj| self.map[adj] > height)
            },
        )
    }

    fn basin_points(&self, low_point: GridPos) -> Vec<GridPos> {
        let mut search_stack: Vec<GridPos> = vec![low_point];

        let mut touched: HashSet<_> = HashSet::new();
        touched.insert(low_point);

        let mut output: Vec<GridPos> = Vec::new();

        while search_stack.len() > 0 {
            let point = search_stack.pop().unwrap();
            if self.map[point] != 9 {
                output.push(point);
                self.adjacent_points(point)
                    .filter(|adj| !touched.contains(&adj))
                    .collect::<Vec<_>>()
                    .iter()
                    .for_each(|adj| {
                        search_stack.push(*adj);
                        touched.insert(*adj);
                    });
            }
        }
        output
    }
}

impl std::str::FromStr for HeightMap {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = s.lines().collect();

        Ok(HeightMap { map })
    }
}

impl Day09 {
    fn parse_height_map(&self) -> Result<HeightMap, Error> {
        // let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let map = puzzle_input.lines().collect();
        Ok(HeightMap { map })
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
            .map(|(_pos, height)| (height + 1) as u64)
            .sum::<u64>();
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let height_map = self.parse_height_map()?;

        let result = height_map
            .low_points()
            .map(|(pos, _height)| height_map.basin_points(pos).len())
            .sorted_by_key(|&i| -(i as i64))
            .take(3)
            .product::<usize>();
        Ok(Box::new(result))
    }
}
