#![allow(unused_imports)]
use crate::utils::graph::DynamicGraph;
use crate::utils::Error;
use crate::utils::{Adjacency, GridMap, GridPos};
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::collections::HashMap;

pub struct Day15;

#[derive(Debug)]
struct RiskMap {
    grid: GridMap<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SearchPointInfo {
    src_to_pos: i64,
    heuristic_to_dest: i64,
    previous_point: Option<GridPos>,
    finalized: bool,
}

impl RiskMap {
    fn enlarge_by(&self, factor: usize) -> Self {
        let grid = self
            .grid
            .iter()
            .map(|(pos, val)| {
                let (x, y) = pos.as_xy(&self.grid);
                (x as usize, y as usize, *val)
            })
            .flat_map(|(x, y, val)| {
                (0..factor).map(move |tile_x| {
                    let x = tile_x * self.grid.x_size + x;
                    let val = (val + (tile_x as u8) - 1) % 9 + 1;
                    (x, y, val)
                })
            })
            .flat_map(|(x, y, val)| {
                (0..factor).map(move |tile_y| {
                    let y = tile_y * self.grid.y_size + y;
                    let val = (val + (tile_y as u8) - 1) % 9 + 1;
                    (x, y, val)
                })
            })
            .collect();
        Self { grid }
    }

    fn path_cost(&self) -> Result<u64, Error> {
        self.shortest_path(self.grid.top_left(), self.grid.bottom_right())
            .map(|edges| edges.into_iter().map(|(_node, weight)| weight).sum())
            .map_err(|_e| Error::NoPathToDest)
    }

    fn adjacent_points(
        &self,
        pos: GridPos,
    ) -> impl Iterator<Item = GridPos> + '_ {
        self.grid.adjacent_points(pos, Adjacency::Rook)
    }
}

impl DynamicGraph<GridPos> for RiskMap {
    fn connections_from(&self, pos: &GridPos) -> Vec<(GridPos, u64)> {
        self.adjacent_points(*pos)
            .map(|adj| (adj, self.grid[adj] as u64))
            .collect()
    }

    fn heuristic_between(&self, a: &GridPos, b: &GridPos) -> Option<u64> {
        let min_cost_per_step = 1;
        Some(min_cost_per_step * (self.grid.manhattan_dist(a, b) as u64))
    }
}

impl Day15 {
    fn parse_inputs(&self) -> Result<RiskMap, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let grid = puzzle_input.lines().collect();
        Ok(RiskMap { grid })
    }
}

impl Puzzle for Day15 {
    fn day(&self) -> i32 {
        15
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let map = self.parse_inputs()?;
        let result = map.path_cost()?;
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let map = self.parse_inputs()?.enlarge_by(5);
        let result = map.path_cost()?;
        Ok(Box::new(result))
    }
}
