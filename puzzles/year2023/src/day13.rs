use std::fmt::Display;

use aoc_utils::prelude::*;

#[derive(Debug)]
pub struct Terrain {
    layers: Vec<Layer>,
}

#[derive(Debug)]
struct Layer(GridMap<Tile>);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Tile {
    Ash,
    Rock,
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Tile::Ash => '.',
            Tile::Rock => '#',
        };
        write!(f, "{c}")
    }
}

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::Ash),
            '#' => Ok(Tile::Rock),
            _ => Err(Error::UnknownChar(c)),
        }
    }
}

impl Display for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Layer {
    fn find_reflection(&self, num_smudges: usize) -> Option<usize> {
        let (width, height) = self.0.shape();

        let x_reflection = (0..width)
            .filter(|&x| self.is_reflection_col(x, num_smudges))
            .map(|x| x + 1);

        let y_reflection = (0..height)
            .filter(|&y| self.is_reflection_row(y, num_smudges))
            .map(|y| (y + 1) * 100);

        std::iter::empty()
            .chain(x_reflection)
            .chain(y_reflection)
            .next()
    }

    fn num_mismatch_across_col(&self, x: usize) -> Option<usize> {
        let map = &self.0;

        let (width, height) = map.shape();
        (0..usize::min(x + 1, width - x - 1))
            .map(move |dx| {
                let x1 = x - dx;
                let x2 = x + 1 + dx;
                (x1, x2)
            })
            .flat_map(move |(x1, x2)| {
                (0..height).map(move |y| {
                    map[(x1 as i64, y as i64)] == map[(x2 as i64, y as i64)]
                })
            })
            .fold(None, |state: Option<usize>, is_match: bool| {
                match (state, is_match) {
                    (None, true) => Some(0),
                    (None, false) => Some(1),
                    (Some(val), b) => Some(val + (!b as usize)),
                }
            })
    }

    fn is_reflection_col(&self, x: usize, num_smudges: usize) -> bool {
        let (width, _) = self.0.shape();
        x + 1 < width
            && self
                .num_mismatch_across_col(x)
                .map(|num| num == num_smudges)
                .unwrap_or(false)
    }

    fn num_mismatch_across_row(&self, y: usize) -> Option<usize> {
        let map = &self.0;

        let (width, height) = map.shape();
        (0..usize::min(y + 1, height - y - 1))
            .map(move |dy| {
                let y1 = y - dy;
                let y2 = y + 1 + dy;
                (y1, y2)
            })
            .flat_map(move |(y1, y2)| {
                (0..width).map(move |x| {
                    map[(x as i64, y1 as i64)] == map[(x as i64, y2 as i64)]
                })
            })
            .fold(None, |state: Option<usize>, is_match: bool| {
                match (state, is_match) {
                    (None, true) => Some(0),
                    (None, false) => Some(1),
                    (Some(val), b) => Some(val + (!b as usize)),
                }
            })
    }

    fn is_reflection_row(&self, y: usize, num_smudges: usize) -> bool {
        let (_, height) = self.0.shape();
        y + 1 < height
            && self
                .num_mismatch_across_row(y)
                .map(|num| num == num_smudges)
                .unwrap_or(false)
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Terrain;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let layers = lines
            .batching(|iter| -> Option<GridMap<Tile>> {
                let first = iter.next()?;
                Some(
                    std::iter::empty::<&str>()
                        .chain(std::iter::once(first))
                        .chain(iter)
                        .take_while(|line| !line.is_empty())
                        .collect(),
                )
            })
            .map(Layer)
            .collect();
        Ok(Terrain { layers })
    }

    fn part_1(
        terrain: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = terrain
            .layers
            .iter()
            .map(|layer| layer.find_reflection(0).unwrap())
            .sum::<usize>();
        Ok(value)
    }

    fn part_2(
        terrain: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = terrain
            .layers
            .iter()
            .map(|layer| layer.find_reflection(1).unwrap())
            .sum::<usize>();
        Ok(value)
    }
}
