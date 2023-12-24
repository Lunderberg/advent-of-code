use std::{collections::HashMap, fmt::Display};

use aoc_utils::{direction::Direction, prelude::*};
use bit_set::BitSet;

#[derive(Clone)]
pub struct ForestMap {
    map: GridMap<Tile>,
    follow_slopes: bool,
}

#[derive(Debug)]
struct Connection {
    dest: usize,
    distance: u64,
}

#[derive(Debug)]
struct ForestGraph {
    key_points: Vec<Vector<2, i64>>,
    index_lookup: HashMap<Vector<2, i64>, usize>,
    connections: Vec<Vec<Connection>>,
}

#[derive(Clone)]
enum Tile {
    Path,
    Forest,
    SlopeLeft,
    SlopeRight,
    SlopeUp,
    SlopeDown,
}

impl Display for ForestMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.map)
    }
}
impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::Path),
            '#' => Ok(Tile::Forest),
            '<' => Ok(Tile::SlopeLeft),
            '>' => Ok(Tile::SlopeRight),
            '^' => Ok(Tile::SlopeUp),
            'v' => Ok(Tile::SlopeDown),
            _ => Err(Error::UnknownChar(c)),
        }
    }
}
impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Tile::Path => '.',
            Tile::Forest => '#',
            Tile::SlopeLeft => '<',
            Tile::SlopeRight => '>',
            Tile::SlopeUp => '^',
            Tile::SlopeDown => 'v',
        };
        write!(f, "{c}")
    }
}

impl Tile {
    fn is_allowed_dir(&self, dir: Direction, follow_slopes: bool) -> bool {
        match (self, dir) {
            (Tile::Path, _) => true,
            (Tile::SlopeLeft, Direction::Left) => true,
            (Tile::SlopeRight, Direction::Right) => true,
            (Tile::SlopeUp, Direction::Up) => true,
            (Tile::SlopeDown, Direction::Down) => true,

            (
                Tile::SlopeLeft
                | Tile::SlopeRight
                | Tile::SlopeUp
                | Tile::SlopeDown,
                _,
            ) if !follow_slopes => true,

            _ => false,
        }
    }
}

impl ForestMap {
    fn allow_movement_against_slope(mut self) -> Self {
        self.follow_slopes = false;
        self
    }

    fn start(&self) -> Vector<2, i64> {
        let pos = [1, 0].into();
        assert!(matches!(self.map.get(pos), Some(Tile::Path)));
        pos
    }

    fn end(&self) -> Vector<2, i64> {
        let pos = self.map.shape_vec() - [2, 1].into();
        assert!(matches!(self.map.get(pos), Some(Tile::Path)));
        pos
    }

    fn is_crossroad(&self, pos: Vector<2, i64>) -> bool {
        matches!(self.map.get(pos), Some(Tile::Path)) && {
            let num_connections = Direction::iter_cardinal()
                .filter(|dir| {
                    self.map
                        .get(pos + dir.as_vec())
                        .map(|tile| !matches!(tile, Tile::Forest))
                        .unwrap_or(false)
                })
                .count();
            num_connections > 2
        }
    }

    fn is_key_point(&self, pos: Vector<2, i64>) -> bool {
        self.is_crossroad(pos) || pos == self.start() || pos == self.end()
    }

    fn key_points(&self) -> impl Iterator<Item = Vector<2, i64>> + '_ {
        let crossroads = self.map.iter().filter(|&pos| self.is_crossroad(pos));
        [self.start(), self.end()].into_iter().chain(crossroads)
    }

    fn reduced_graph(&self) -> ForestGraph {
        let key_points: Vec<_> = self.key_points().collect();
        let index_lookup: HashMap<_, _> = key_points
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, pos)| (pos, i))
            .collect();

        let connections: Vec<Vec<Connection>> = key_points
            .iter()
            .map(|&key_point| -> Vec<Connection> {
                // println!("Starting from key point {key_point}");
                let conns = self
                    .iter_dijkstra([(key_point, !self.is_crossroad(key_point))])
                    .filter(|search_item| search_item.item.0 != key_point)
                    .filter(|search_item| self.is_key_point(search_item.item.0))
                    .filter(|search_item| search_item.item.1)
                    .map(|search_item| Connection {
                        dest: index_lookup[&search_item.item.0],
                        distance: search_item.total_dist,
                    })
                    .collect();
                conns
            })
            .collect();

        ForestGraph {
            key_points,
            index_lookup,
            connections,
        }
    }
}

impl ForestGraph {
    fn iter_paths(
        &self,
        initial: Vector<2, i64>,
    ) -> impl Iterator<Item = (Vector<2, i64>, u64)> + '_ {
        let initial = *self.index_lookup.get(&initial).unwrap();
        let mut to_visit: Vec<_> =
            vec![(initial, std::iter::once(initial).collect(), 0)];

        std::iter::from_fn(move || {
            let (visiting, visited, dist): (usize, BitSet<usize>, u64) =
                to_visit.pop()?;

            self.connections
                .get(visiting)
                .into_iter()
                .flatten()
                .filter(|conn| !visited.contains(conn.dest))
                .for_each(|conn| {
                    to_visit.push((
                        conn.dest,
                        visited
                            .iter()
                            .chain(std::iter::once(visiting))
                            .collect(),
                        dist + conn.distance,
                    ));
                });

            Some((self.key_points[visiting], dist))
        })
    }
}

impl EdgeWeightedGraph<(Vector<2, i64>, bool)> for ForestMap {
    fn connections_from<'a>(
        &'a self,
        (old_pos, seen_crossroad): &'a (Vector<2, i64>, bool),
    ) -> impl Iterator<Item = ((Vector<2, i64>, bool), u64)> + '_ {
        let seen_crossroad = *seen_crossroad;
        let is_crossroad = self.is_crossroad(*old_pos);

        Direction::iter_cardinal()
            .filter(move |_| !(seen_crossroad && is_crossroad))
            .filter_map(|dir| {
                let new_pos = *old_pos + dir.as_vec();
                self.map
                    .get(new_pos)
                    .filter(|tile| tile.is_allowed_dir(dir, self.follow_slopes))
                    .map(|_| new_pos)
            })
            .map(move |new_pos| ((new_pos, seen_crossroad || is_crossroad), 1))
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = ForestMap;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let map = lines.collect();
        Ok(ForestMap {
            map,
            follow_slopes: true,
        })
    }

    fn part_1(
        forest: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let reduced_graph = forest.reduced_graph();

        let longest_path = reduced_graph
            .iter_paths(forest.start())
            .filter(|(pos, _)| *pos == forest.end())
            .map(|(_, dist)| dist)
            // .inspect(|dist| println!("Distance {dist} to end of path"))
            .max()
            .unwrap();

        Ok(longest_path)
    }

    fn part_2(
        forest: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let reduced_graph = forest
            .clone()
            .allow_movement_against_slope()
            .reduced_graph();

        let longest_path = reduced_graph
            .iter_paths(forest.start())
            .filter(|(pos, _)| *pos == forest.end())
            .map(|(_, dist)| dist)
            // .inspect(|dist| println!("Distance {dist} to end of path"))
            .max()
            .unwrap();

        Ok(longest_path)
    }
}
