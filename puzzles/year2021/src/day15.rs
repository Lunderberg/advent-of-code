use aoc_utils::prelude::*;

use crate::utils::Adjacency;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

#[derive(Debug)]
pub struct RiskMap {
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
            .map(|((x, y), &val): ((i64, i64), _)| {
                (x as usize, y as usize, val)
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

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = RiskMap;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(RiskMap {
            grid: lines.collect(),
        })
    }

    fn part_1(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        parsed.path_cost()
    }

    fn part_2(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        parsed.enlarge_by(5).path_cost()
    }
}
