use aoc_utils::prelude::*;

use crate::utils::Adjacency;

use std::fmt::{Display, Formatter};

type Point = Vector<2, i64>;

#[derive(Debug, Clone)]
pub struct ElfSystem {
    map: GridMap<Tile>,
    next_step: ElfStep,
    iteration: usize,
}

#[derive(Debug, Clone)]
enum ElfStep {
    Proposal,
    Application,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tile {
    Elf,
    Ground,
    ValidProposal,
    ConflictingProposal,
}

impl Direction {
    fn offset(&self) -> Vector<2, i64> {
        match self {
            Direction::North => [0, -1].into(),
            Direction::South => [0, 1].into(),
            Direction::East => [1, 0].into(),
            Direction::West => [-1, 0].into(),
        }
    }

    fn offsets(&self) -> impl Iterator<Item = Vector<2, i64>> {
        let offset = self.offset();
        let step = match self {
            Direction::North | Direction::South => Direction::East.offset(),
            Direction::East | Direction::West => Direction::North.offset(),
        };
        (-1..=1).map(move |scale| offset + scale * step)
    }
}

impl ElfSystem {
    fn next(&self) -> Self {
        match self.next_step {
            ElfStep::Proposal => self.make_proposals(),
            ElfStep::Application => self.apply_proposals(),
        }
    }

    fn checked_directions(&self) -> impl Iterator<Item = Direction> {
        vec![
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ]
        .into_iter()
        .cycle()
        .skip(self.iteration.rem_euclid(4))
        .take(4)
    }

    fn proposed_location(&self, elf: Point) -> Option<Point> {
        let has_neighbor = self
            .map
            .adjacent_values_default(elf, Adjacency::Queen, Tile::Ground)
            .any(|tile| matches!(&tile, Tile::Elf));

        has_neighbor
            .then(|| {
                self.checked_directions()
                    .find(|dir| {
                        dir.offsets().all(|offset| {
                            let to_check: Point = elf + offset;
                            self.map
                                .get(to_check)
                                .map(|tile| !matches!(tile, Tile::Elf))
                                .unwrap_or(true)
                        })
                    })
                    .map(|dir| elf + dir.offset())
            })
            .flatten()
    }

    fn make_proposals(&self) -> Self {
        let map = self
            .map
            .iter_vec()
            .filter(|(_, tile)| matches!(tile, Tile::Elf))
            .map(|(pos, _)| pos)
            .filter_map(|pos| self.proposed_location(pos))
            .counts()
            .into_iter()
            .map(|(pos, count)| match count {
                1 => (pos, Tile::ValidProposal),
                _ => (pos, Tile::ConflictingProposal),
            })
            .chain(
                self.map
                    .iter_vec()
                    .filter(|(_pos, tile)| matches!(tile, Tile::Elf))
                    .map(|(pos, tile)| (pos, tile.clone())),
            )
            .collect_resized_grid_map(Tile::Ground);

        Self {
            map,
            iteration: self.iteration,
            next_step: ElfStep::Application,
        }
    }

    fn apply_proposals(&self) -> Self {
        let map = self
            .map
            .iter_vec()
            .filter(|(_, tile)| matches!(tile, Tile::Elf))
            .map(|(pos, _)| pos)
            .map(|pos| {
                self.proposed_location(pos)
                    .filter(|proposed| {
                        self.map.is_valid(*proposed)
                            && matches!(
                                self.map[*proposed],
                                Tile::ValidProposal
                            )
                    })
                    .unwrap_or(pos)
            })
            .map(|pos| (pos, Tile::Elf))
            .collect_resized_grid_map(Tile::Ground);

        Self {
            map,
            iteration: self.iteration + 1,
            next_step: ElfStep::Proposal,
        }
    }

    fn empty_ground_in_bounding_box(&self) -> usize {
        let total = self.map.x_size * self.map.y_size;
        let num_elves = self
            .map
            .iter()
            .filter(|(_, tile)| matches!(tile, Tile::Elf))
            .count();
        total - num_elves
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let as_string = match self {
            Direction::North => "North",
            Direction::South => "South",
            Direction::East => "East",
            Direction::West => "West",
        };
        write!(f, "{as_string}")
    }
}

impl Display for ElfSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.map)
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Tile::Elf => '#',
            Tile::Ground => '.',
            Tile::ValidProposal => 'P',
            Tile::ConflictingProposal => 'X',
        };
        write!(f, "{c}")
    }
}
impl std::str::FromStr for Tile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().exactly_one_or_err()? {
            '#' => Ok(Tile::Elf),
            '.' => Ok(Tile::Ground),
            c => Err(Error::UnknownChar(c)),
        }
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = ElfSystem;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let map = lines.collect();
        Ok(ElfSystem {
            map,
            next_step: ElfStep::Proposal,
            iteration: 0,
        })
    }

    fn part_1(
        elves: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(std::iter::successors(Some(elves.clone()), |elves| {
            Some(elves.next())
        })
        .find(|elves| elves.iteration == 10)
        .unwrap()
        .empty_ground_in_bounding_box())
    }

    fn part_2(
        elves: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(std::iter::successors(Some(elves.clone()), |elves| {
            Some(elves.next())
        })
        .step_by(2)
        .tuple_windows()
        .find(|(a, b)| a.map == b.map)
        .map(|(_, elves)| elves.iteration)
        .unwrap())
    }
}
