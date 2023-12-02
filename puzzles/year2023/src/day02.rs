use std::str::FromStr;

use aoc_utils::prelude::*;

#[derive(Debug)]
pub struct Game {
    id: u32,
    sets: Vec<CubeSet>,
}

#[derive(Debug, Default)]
struct CubeSet {
    red: u32,
    green: u32,
    blue: u32,
}

impl FromStr for Game {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (game, sets) = s
            .split(":")
            .collect_tuple()
            .ok_or(Error::WrongIteratorSize)?;

        let (_, id) = game
            .split_ascii_whitespace()
            .collect_tuple()
            .ok_or(Error::WrongIteratorSize)?;
        let id = id.parse()?;

        let sets = sets
            .split(";")
            .map(|set_str| set_str.parse())
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(Game { id, sets })
    }
}
impl FromStr for CubeSet {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut red = None;
        let mut green = None;
        let mut blue = None;

        for item in s.split(",") {
            let (count, color) = item
                .trim()
                .split_ascii_whitespace()
                .collect_tuple()
                .ok_or(Error::WrongIteratorSize)?;
            let count: u32 = count.parse()?;

            let out: &mut Option<u32> = match color {
                "red" => &mut red,
                "green" => &mut green,
                "blue" => &mut blue,

                _ => {
                    return Err(Error::InvalidString(color.to_string()));
                }
            };

            if out.is_some() {
                return Err(Error::TooManyValues);
            } else {
                *out = Some(count);
            }
        }

        Ok(CubeSet {
            red: red.unwrap_or(0),
            green: green.unwrap_or(0),
            blue: blue.unwrap_or(0),
        })
    }
}

impl Game {
    fn part_1_legal(&self) -> bool {
        self.sets
            .iter()
            .all(|set| set.red <= 12 && set.green <= 13 && set.blue <= 14)
    }

    fn min_set(&self) -> CubeSet {
        self.sets.iter().fold(Default::default(), |a, b| CubeSet {
            red: a.red.max(b.red),
            green: a.green.max(b.green),
            blue: a.blue.max(b.blue),
        })
    }
}
impl CubeSet {
    fn part_2_power(&self) -> u32 {
        self.red * self.blue * self.green
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<Game>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        games: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = games
            .iter()
            .filter(|game| game.part_1_legal())
            .map(|game| game.id)
            .sum::<u32>();
        Ok(value)
    }

    fn part_2(
        games: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = games
            .iter()
            .map(|game| game.min_set())
            .map(|min_set| min_set.part_2_power())
            .sum::<u32>();
        Ok(value)
    }
}
