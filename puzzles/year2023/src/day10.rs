use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::{Index, IndexMut},
};

use aoc_utils::prelude::*;
use console::Style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, Copy)]
struct Tile {
    connections: [bool; 4],
    has_animal: bool,
}

#[derive(Debug, Clone)]
pub struct PipeMap(GridMap<Tile>);

impl From<Direction> for usize {
    fn from(val: Direction) -> Self {
        match val {
            Direction::North => 0,
            Direction::South => 1,
            Direction::East => 2,
            Direction::West => 3,
        }
    }
}
impl From<Direction> for Vector<2, i64> {
    fn from(val: Direction) -> Self {
        match val {
            Direction::North => [0, -1].into(),
            Direction::South => [0, 1].into(),
            Direction::East => [1, 0].into(),
            Direction::West => [-1, 0].into(),
        }
    }
}

impl Index<Direction> for Tile {
    type Output = bool;

    fn index(&self, index: Direction) -> &Self::Output {
        let index: usize = index.into();
        &self.connections[index]
    }
}
impl IndexMut<Direction> for Tile {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        let index: usize = index.into();
        &mut self.connections[index]
    }
}

impl Tile {
    const GROUND: Self = Self {
        connections: [false; 4],
        has_animal: false,
    };
    const ANIMAL: Self = Self {
        connections: [false; 4],
        has_animal: true,
    };

    fn from_directions(a: Direction, b: Direction) -> Self {
        let mut output = Self::GROUND;
        output[a] = true;
        output[b] = true;
        output
    }
}

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '|' => {
                Ok(Tile::from_directions(Direction::North, Direction::South))
            }
            '-' => Ok(Tile::from_directions(Direction::East, Direction::West)),
            'L' => Ok(Tile::from_directions(Direction::North, Direction::East)),
            'J' => Ok(Tile::from_directions(Direction::North, Direction::West)),
            '7' => Ok(Tile::from_directions(Direction::West, Direction::South)),
            'F' => Ok(Tile::from_directions(Direction::East, Direction::South)),
            'S' => Ok(Tile::ANIMAL),
            '.' => Ok(Tile::GROUND),
            _ => Ok(Tile::GROUND),
            // _ => Err(Error::UnknownChar(value)),
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Tile {
                has_animal: true, ..
            } => 'S',
            Tile {
                connections: [false, false, false, false],
                ..
            } => '.',
            Tile {
                connections: [false, false, true, true],
                ..
            } => '-',
            Tile {
                connections: [false, true, false, true],
                ..
            } => '7',
            Tile {
                connections: [false, true, true, false],
                ..
            } => 'F',
            Tile {
                connections: [true, false, false, true],
                ..
            } => 'J',
            Tile {
                connections: [true, false, true, false],
                ..
            } => 'L',
            Tile {
                connections: [true, true, false, false],
                ..
            } => '|',
            _ => {
                panic!("Invalid tile")
            }
        };
        write!(f, "{c}")
    }
}

impl Display for PipeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Direction {
    fn iter() -> impl Iterator<Item = Direction> {
        [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ]
        .into_iter()
    }

    fn reverse(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

impl PipeMap {
    fn animal_location(&self) -> Option<Vector<2, i64>> {
        self.0
            .iter()
            .find(|(_, tile): &(_, &Tile)| tile.has_animal)
            .map(|(pos, _)| pos)
    }

    fn infer_animal(&self) -> Option<(Vector<2, i64>, Tile)> {
        self.0
            .iter()
            .find(|(_, tile): &(Vector<2, i64>, &Tile)| tile.has_animal)
            .map(|(pos, _)| {
                let (a, b) = Direction::iter()
                    .filter(|&dir| {
                        self.0
                            .get(pos + dir.into())
                            .map(|loc| loc[dir.reverse()])
                            .unwrap_or(false)
                    })
                    .collect_tuple()
                    .expect(
                        "Invalid number of adjacent pipes pointing to animal",
                    );
                let mut tile = Tile::ANIMAL;
                tile[a] = true;
                tile[b] = true;
                (pos, tile)
            })
    }

    fn with_inferred_animal(mut self) -> Self {
        if let Some((pos, tile)) = self.infer_animal() {
            self.0[pos] = tile;
        }
        self
    }
}

impl EdgeWeightedGraph<Vector<2, i64>> for PipeMap {
    fn connections_from<'a>(
        &'a self,
        node: &'a Vector<2, i64>,
    ) -> impl Iterator<Item = (Vector<2, i64>, u64)> + '_ {
        let node = *node;
        let current_tile = self.0[node];
        Direction::iter()
            .filter(move |&dir| current_tile[dir])
            .map(move |dir| (node + dir.into(), 1))
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    // const EXAMPLE_NUM: u8 = 0; // Part 1, Square loop
    // const EXAMPLE_NUM: u8 = 3; // Part 1, bigger loop
    // const EXAMPLE_NUM: u8 = 9; // Part 2, Wide path
    // const EXAMPLE_NUM: u8 = 11; // Part 2, Narrow path
    // const EXAMPLE_NUM: u8 = 12; // Part 2, wind-y example
    const EXAMPLE_NUM: u8 = 14; // Part 2, enclosed pipes

    type ParsedInput = PipeMap;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(PipeMap(lines.collect()))
    }

    fn part_1(map: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        let map = map.clone().with_inferred_animal();

        let connected: HashSet<_> = map
            .iter_dijkstra(map.animal_location())
            .map(|node| node.item)
            .collect();
        let styled_map = map.0.map(|(pos, tile)| {
            (if connected.contains(&pos) {
                Style::new().green().bright().bold()
            } else {
                Style::new().red()
            })
            .apply_to(tile)
        });
        println!("Map:\n{styled_map}");

        let distance = map
            .iter_dijkstra(map.animal_location())
            .last()
            .map(|item| item.total_dist)
            .unwrap();
        Ok(distance)
    }

    fn part_2(map: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        let map = map.clone().with_inferred_animal();

        let start_loc = map.animal_location().unwrap();
        let start_dir_a = Direction::iter().find(|&dir| map.0[start_loc][dir])
            .expect("No viable start direction");

        let loop_winding: HashMap<_, _> = std::iter::successors(
            Some((start_loc, Direction::North, start_dir_a)),
            |&(loc, _, in_dir)| {
                let loc = loc + in_dir.into();
                let tile = &map.0[loc];
                let out_dir = Direction::iter().find(|&dir| dir != in_dir.reverse() && tile[dir])
                    .expect("No output direction found");
                Some((loc, in_dir, out_dir))
            },
        )
        .skip(1)
        .take_while_inclusive(|&(loc, _, _)| loc != start_loc)
        .map(|(loc, in_dir, out_dir)| -> (_, i64) {
            // Moving to the right, in the top half of the tile.
            let winding = match (in_dir, out_dir) {
                (Direction::South, Direction::North)
                | (Direction::East, Direction::West)
                | (Direction::West, Direction::East)
                | (Direction::North, Direction::South) => {
                    panic!("Invalid in/out pair {in_dir:?}/{out_dir:?}")
                }

                (_, Direction::North) => 1,
                (Direction::South, _) => -1,
                _ => 0,
            };
            // println!(
            //     "Location {loc}, \
            //      entered heading {in_dir:?} ({}) \
            //      and exiting heading {out_dir:?} ({}), \
            //      has winding {winding}",
            //     Into::<Vector<2, i64>>::into(in_dir),
            //     Into::<Vector<2, i64>>::into(out_dir),
            // );
            (loc, winding)
        })
        .collect();

        let contained: HashMap<_, _> = map
            .0
            .iter()
            .scan(
                0,
                |state: &mut i64,
                 loc: Vector<2, i64>|
                 -> Option<(Vector<2, i64>, i64)> {
                    if loc.x() == 0 {
                        // println!("");
                        *state = 0;
                    }
                    let tile_winding =
                        loop_winding.get(&loc).cloned().unwrap_or(0);
                    *state += tile_winding;
                    // println!(
                    //     "Location {loc} \
                    //      has local winding of {tile_winding} \
                    //      so the cumulative (per row) is {}",
                    //     *state
                    // );
                    Some((loc, *state))
                },
            )
            .filter(|(loc, total_winding)| {
                !loop_winding.contains_key(loc) && *total_winding != 0
            })
            .collect();

        let styled_map = map.0.map(|(pos, tile)| {
            let style = if contained.contains_key(&pos) {
                Style::new().blue().bright().bold()
            } else if loop_winding.contains_key(&pos) {
                Style::new().green().bright().bold()
            } else {
                Style::new().red()
            };
            style.apply_to(tile)
        });
        println!("Map:\n{styled_map}");

        Ok(contained.len())
    }
}
