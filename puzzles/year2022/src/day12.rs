use aoc_utils::prelude::*;

use crate::utils::graph::SearchResult;
use crate::utils::Adjacency;

use std::collections::HashSet;
use std::fmt::{Display, Formatter};

pub struct HeightMap {
    map: GridMap<MapChar>,
}

#[derive(Clone, Copy)]
enum MapChar {
    Height(i8),
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum ReverseSearchPos {
    LowestHeight,
    Pos(GridPos),
}

impl TryFrom<char> for MapChar {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'a'..='z' => Ok(MapChar::Height(((c as u32) - ('a' as u32)) as i8)),
            'S' => Ok(MapChar::Start),
            'E' => Ok(MapChar::End),
            _ => Err(Error::UnknownChar(c)),
        }
    }
}

impl Display for MapChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let c = match self {
            MapChar::Height(val) => {
                char::from_u32((*val as u32) + ('a' as u32)).unwrap()
            }
            MapChar::Start => 'S',
            MapChar::End => 'E',
        };

        write!(f, "{c}")
    }
}

impl Display for HeightMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.map)
    }
}

impl MapChar {
    fn height(&self) -> i8 {
        match self {
            MapChar::Start => 0,
            MapChar::End => 25,
            MapChar::Height(val) => *val,
        }
    }
}

impl HeightMap {
    fn start(&self) -> Result<GridPos, Error> {
        self.map
            .iter()
            .find(|(_pos, &val)| matches!(val, MapChar::Start))
            .map(|(pos, _val)| pos)
            .ok_or(Error::NoStartPosition)
    }
    fn end(&self) -> Result<GridPos, Error> {
        self.map
            .iter()
            .find(|(_pos, &val)| matches!(val, MapChar::End))
            .map(|(pos, _val)| pos)
            .ok_or(Error::NoEndPosition)
    }
    fn highlight(&self, highlight: HashSet<GridPos>) -> String {
        let highlighted: GridMap<String> = self
            .map
            .iter_pos()
            .map(|(grid_pos, value)| {
                let (x, y) = grid_pos.as_xy(&self.map);
                (
                    x as usize,
                    y as usize,
                    if highlight.contains(&grid_pos) {
                        format!("\x1b[92m{value}\x1b[0m")
                    } else {
                        format!("\x1b[91m{value}\x1b[0m")
                    },
                )
            })
            .collect();
        format!("{highlighted}")
    }
}

impl DynamicGraph<GridPos> for HeightMap {
    fn connections_from(&self, node: &GridPos) -> Vec<(GridPos, u64)> {
        let current_height = self.map[*node].height();
        self.map
            .adjacent_points(*node, Adjacency::Rook)
            .filter(|point| self.map[*point].height() - current_height <= 1)
            .map(|point| (point, 1))
            .collect()
    }

    fn heuristic_between(
        &self,
        node_from: &GridPos,
        node_to: &GridPos,
    ) -> Option<u64> {
        Some(self.map.manhattan_dist(node_from, node_to) as u64)
    }
}

impl DynamicGraph<ReverseSearchPos> for HeightMap {
    fn connections_from(
        &self,
        node: &ReverseSearchPos,
    ) -> Vec<(ReverseSearchPos, u64)> {
        match node {
            ReverseSearchPos::LowestHeight => Vec::new(),
            ReverseSearchPos::Pos(pos) => {
                let current_height = self.map[*pos].height();
                self.map
                    .adjacent_points(*pos, Adjacency::Rook)
                    .filter(|point| {
                        current_height - self.map[*point].height() <= 1
                    })
                    .map(|point| {
                        let p = if self.map[point].height() == 0 {
                            ReverseSearchPos::LowestHeight
                        } else {
                            ReverseSearchPos::Pos(point)
                        };
                        (p, 1)
                    })
                    .collect()
            }
        }
    }

    fn heuristic_between(
        &self,
        node_from: &ReverseSearchPos,
        node_to: &ReverseSearchPos,
    ) -> Option<u64> {
        let height_from = match node_from {
            ReverseSearchPos::LowestHeight => 0,
            ReverseSearchPos::Pos(pos) => self.map[*pos].height(),
        };
        let height_to = match node_to {
            ReverseSearchPos::LowestHeight => 0,
            ReverseSearchPos::Pos(pos) => self.map[*pos].height(),
        };
        Some(height_from.abs_diff(height_to) as u64)
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = HeightMap;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let map = lines.collect();
        Ok(HeightMap { map })
    }

    fn part_1(map: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        let start = map.start()?;
        let end = map.end()?;

        // println!("Map:\n{map}");
        // println!(
        //     "Find route from {:?} to {:?}",
        //     start.as_xy(&map.map),
        //     end.as_xy(&map.map)
        // );

        let res = map.shortest_path_search_result(start, end);

        match res {
            SearchResult::Success { path } => {
                let highlighted =
                    map.highlight(path.iter().map(|(pos, _)| *pos).collect());
                println!("Path:\n{highlighted}");
                Ok(path.len())
            }
            SearchResult::HeuristicFailsOnStartPoint => {
                println!("Heuristic fail");
                Err(Error::NoPathToDest)
            }
            SearchResult::NoPathToTarget { reachable } => {
                println!(
                    "Reachable:\n{}",
                    map.highlight(reachable.into_iter().collect())
                );
                Err(Error::NoPathToDest)
            }
            SearchResult::OtherError(err) => {
                println!("Other error");
                Err(err.into())
            }
        }

        //let path = map.shortest_path(start, end)?;

        //println!("Path: {path:?}");

        //Ok(path.len())
    }

    fn part_2(map: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        let peak = map.end()?;

        let res = map.shortest_path_search_result(
            ReverseSearchPos::Pos(peak),
            ReverseSearchPos::LowestHeight,
        );
        match res {
            SearchResult::Success { path } => {
                let highlighted = map.highlight(
                    path.iter()
                        .filter_map(|(pos, _)| match pos {
                            ReverseSearchPos::Pos(pos) => Some(*pos),
                            _ => None,
                        })
                        .collect(),
                );
                println!("Path:\n{highlighted}");
                Ok(path.len())
            }
            SearchResult::HeuristicFailsOnStartPoint => {
                println!("Heuristic fail");
                Err(Error::NoPathToDest)
            }
            SearchResult::NoPathToTarget { reachable } => {
                println!(
                    "Reachable:\n{}",
                    map.highlight(
                        reachable
                            .into_iter()
                            .filter_map(|pos| {
                                match pos {
                                    ReverseSearchPos::Pos(pos) => Some(pos),
                                    _ => None,
                                }
                            })
                            .collect()
                    )
                );
                Err(Error::NoPathToDest)
            }
            SearchResult::OtherError(err) => {
                println!("Other error");
                Err(err.into())
            }
        }
    }
}
