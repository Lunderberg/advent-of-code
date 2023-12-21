use std::{collections::HashSet, fmt::Display};

use aoc_utils::direction::Direction;
use aoc_utils::{prelude::*, IntoGridPos};

pub struct GardenMap(GridMap<Tile>);

#[derive(Debug, Clone, Copy)]
enum Tile {
    Rock,
    Garden,
    Elf,
}

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::Garden),
            '#' => Ok(Tile::Rock),
            'S' => Ok(Tile::Elf),
            _ => Err(Error::UnknownChar(c)),
        }
    }
}
impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Tile::Rock => '#',
            Tile::Garden => '.',
            Tile::Elf => 'S',
        };
        write!(f, "{c}")
    }
}
impl Display for GardenMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Tile {
    fn is_garden(&self) -> bool {
        matches!(self, Tile::Garden | Tile::Elf)
    }
    fn is_elf(&self) -> bool {
        matches!(self, Tile::Elf)
    }
}

impl GardenMap {
    fn elf_location(&self) -> Option<Vector<2, i64>> {
        self.0
            .iter()
            .find(|(_, tile): &(_, &_)| tile.is_elf())
            .map(|(pos, _)| pos)
    }
    fn without_elf(&self) -> Self {
        let map = self
            .0
            .iter()
            .map(|(pos, &tile)| {
                (pos, if tile.is_elf() { Tile::Garden } else { tile })
            })
            .collect();
        Self(map)
    }
    fn is_garden_tile(&self, pos: impl IntoGridPos) -> bool {
        self.0
            .get(pos)
            .map(|tile| tile.is_garden())
            .unwrap_or(false)
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = GardenMap;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(GardenMap(lines.collect()))
    }

    fn part_1(
        garden: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let initial = garden.elf_location().unwrap();
        let garden = garden.without_elf();

        // If this doesn't work, could use Dijkstra's to find all
        // tiles reachable in less than 64 tiles, count the number of
        // tiles with an even path length.  Only stepping in cardinal
        // directions means that all cycles have an even number of
        // steps.  Therefore, a tile is reachable in on even step
        // counts iff the shortest path arrives on an even step count.
        let num_final_locations = std::iter::successors(
            Some(std::iter::once(initial).collect()),
            |prev: &HashSet<_>| {
                Some(
                    prev.iter()
                        .flat_map(|old_pos| {
                            Direction::iter_cardinal()
                                .map(move |dir| *old_pos + dir.as_vec())
                        })
                        .filter(|&new_pos| garden.is_garden_tile(new_pos))
                        .collect(),
                )
            },
        )
        .nth(64)
        .unwrap()
        .len();

        Ok(num_final_locations)
    }

    fn part_2(_: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        Err::<(), _>(Error::NotYetImplemented)
    }
}
