#![allow(unused_imports)]
use crate::utils::geometry::Vector;
use crate::utils::graph::*;
use crate::{Error, Puzzle};

use itertools::Itertools;

use std::collections::HashSet;
use std::convert::TryInto;
use std::ops::RangeInclusive;

type Point = Vector<3>;

pub struct Lava {
    voxels: HashSet<Point>,
    bounds: [RangeInclusive<i64>; 3],
}

impl Lava {
    fn new(voxels: HashSet<Point>) -> Self {
        let min = voxels
            .iter()
            .cloned()
            .reduce(|a, b| a.zip_map(b, |&ai, &bi| ai.min(bi)))
            .unwrap();
        let max = voxels
            .iter()
            .cloned()
            .reduce(|a, b| a.zip_map(b, |&ai, &bi| ai.max(bi)))
            .unwrap();
        let bounds = min
            .iter()
            .zip(max.iter())
            .map(|(a, b)| (a - 1)..=(b + 1))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Lava { voxels, bounds }
    }

    fn min(&self) -> Point {
        self.bounds.clone().map(|range| *range.start()).into()
    }

    fn faces() -> impl Iterator<Item = Point> {
        vec![
            [0, 0, 1].into(),
            [0, 0, -1].into(),
            [0, 1, 0].into(),
            [0, -1, 0].into(),
            [1, 0, 0].into(),
            [-1, 0, 0].into(),
        ]
        .into_iter()
    }
}

impl DynamicGraph<Point> for Lava {
    fn connections_from(&self, node: &Point) -> Vec<(Point, u64)> {
        Self::faces()
            .map(|offset| *node + offset)
            .filter(|loc| {
                loc.iter()
                    .zip(self.bounds.iter())
                    .all(|(val, bound)| bound.contains(&val))
            })
            .filter(|loc| !self.voxels.contains(loc))
            .map(|loc| (loc, 1))
            .collect()
    }

    fn heuristic_between(
        &self,
        node_from: &Point,
        node_to: &Point,
    ) -> Option<u64> {
        Some(node_from.manhattan_dist(node_to) as u64)
    }
}

pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 18;

    type ParsedInput = Lava;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let voxels =
            lines.map(|line| line.parse()).collect::<Result<_, _>>()?;
        Ok(Lava::new(voxels))
    }

    type Part1Result = usize;
    fn part_1(lava: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        let adjacent = lava
            .voxels
            .iter()
            .tuple_combinations()
            .filter(|(a, b)| a.manhattan_dist(b) == 1)
            .count();

        Ok(6 * lava.voxels.len() - 2 * adjacent)
    }

    type Part2Result = usize;
    fn part_2(lava: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        let externally_reachable: HashSet<Point> = lava
            .dijkstra_paths(lava.min())
            .into_iter()
            .map(|(node, _info)| node)
            .collect();

        let external_faces = lava
            .voxels
            .iter()
            .flat_map(|loc| Lava::faces().map(move |offset| *loc + offset))
            .filter(|loc| externally_reachable.contains(loc))
            .count();

        Ok(external_faces)
    }
}
