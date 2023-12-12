use std::{collections::HashSet, fmt::Display};

use aoc_utils::prelude::*;

#[derive(Debug, Clone, Copy)]
enum Tile {
    Galaxy,
    Empty,
}

pub struct Observation(GridMap<Tile>);

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '#' => Ok(Tile::Galaxy),
            '.' => Ok(Tile::Empty),
            _ => Err(Error::UnknownChar(value)),
        }
    }
}
impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Tile::Galaxy => '#',
            Tile::Empty => '.',
        };
        write!(f, "{c}")
    }
}

impl Observation {
    fn iter_pos(&self) -> impl Iterator<Item = Vector<2, i64>> + '_ {
        self.0
            .iter()
            .filter(|&(_, &tile)| matches!(tile, Tile::Galaxy))
            .map(|(pos, _)| pos)
    }

    fn empty_rows(&self) -> impl Iterator<Item = i64> {
        let with_galaxy: HashSet<_> =
            self.iter_pos().map(|pos| pos.y()).collect();
        let (_, height) = self.0.shape();
        let height = height as i64;
        (0..height).filter(move |row| !with_galaxy.contains(row))
    }

    fn empty_cols(&self) -> impl Iterator<Item = i64> {
        let with_galaxy: HashSet<_> =
            self.iter_pos().map(|pos| pos.x()).collect();
        let (width, _) = self.0.shape();
        let width = width as i64;
        (0..width).filter(move |col| !with_galaxy.contains(col))
    }

    /// Iterator returning the cartesian distance of the observed
    /// locations, and the number of empty rows/columns between the
    /// observed locations.
    fn pairwise_distances(&self) -> impl Iterator<Item = (u64, u64)> + '_ {
        let empty_rows: HashSet<_> = self.empty_rows().collect();
        let empty_cols: HashSet<_> = self.empty_cols().collect();

        self.iter_pos()
            .collect_vec()
            .into_iter()
            .tuple_combinations()
            .map(move |(a, b)| {
                let num_empty = std::iter::empty()
                    .chain(
                        (a.x().min(b.x())..=a.x().max(b.x()))
                            .filter(|x| empty_cols.contains(x)),
                    )
                    .chain(
                        (a.y().min(b.y())..=a.y().max(b.y()))
                            .filter(|y| empty_rows.contains(y)),
                    )
                    .count() as u64;

                let observed_distance = (a - b)
                    .map(|delta| delta.abs() as u64)
                    .into_iter()
                    .sum::<u64>();
                (observed_distance, num_empty)
            })
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Observation;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(Observation(lines.collect()))
    }

    fn part_1(
        image: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let distance =
            image.pairwise_distances().map(|(a, b)| a + b).sum::<u64>();

        Ok(distance)
    }

    fn part_2(
        image: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let distance = image
            .pairwise_distances()
            .map(|(a, b)| a + b * 999999)
            .sum::<u64>();

        Ok(distance)
    }
}
