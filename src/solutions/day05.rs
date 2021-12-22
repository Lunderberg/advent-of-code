#![allow(unused_imports)]
use regex::Regex;

use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

pub struct Day05;

#[derive(Debug, PartialEq, Eq, Hash)]
struct Pos {
    x: i64,
    y: i64,
}

#[derive(Debug)]
pub struct VentLine {
    start: Pos,
    stop: Pos,
}

impl VentLine {
    fn is_diagonal(&self) -> bool {
        (self.start.x != self.stop.x) && (self.start.y != self.stop.y)
    }

    fn vent_locations(&self) -> impl Iterator<Item = Pos> {
        let x_range = (self.start.x - self.stop.x).abs();
        let y_range = (self.start.y - self.stop.y).abs();
        let num_vents = x_range.max(y_range) + 1;
        let dx = (self.stop.x - self.start.x).signum();
        let dy = (self.stop.y - self.start.y).signum();
        let x_init = self.start.x;
        let y_init = self.start.y;
        (0..num_vents).map(move |i| Pos {
            x: x_init + i * dx,
            y: y_init + i * dy,
        })
    }
}

impl Day05 {}

impl Puzzle for Day05 {
    const DAY: u8 = 5;
    const IMPLEMENTED: bool = true;
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<VentLine>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let reg = Regex::new(
            r"^(?P<x1>[0-9]+),(?P<y1>[0-9]+) -> (?P<x2>[0-9]+),(?P<y2>[0-9]+)$",
        )
        .unwrap();

        Ok(lines
            .map(|line| -> Result<_, Error> {
                let captures = reg.captures(line).ok_or(Error::Mismatch)?;
                let vals = vec!["x1", "y1", "x2", "y2"]
                    .iter()
                    .map(|name| {
                        captures
                            .name(name)
                            .unwrap()
                            .as_str()
                            .parse::<i64>()
                            .unwrap()
                    })
                    .collect::<Vec<_>>();
                Ok(VentLine {
                    start: Pos {
                        x: vals[0],
                        y: vals[1],
                    },
                    stop: Pos {
                        x: vals[2],
                        y: vals[3],
                    },
                })
            })
            .collect::<Result<Vec<_>, _>>()?)
    }

    type Part1Result = usize;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        Ok(parsed
            .iter()
            .filter(|vent_line| !vent_line.is_diagonal())
            .map(|vent_line| vent_line.vent_locations())
            .flatten()
            .counts()
            .into_iter()
            .filter(|(_loc, num_occurrences)| *num_occurrences > 1)
            .count())
    }

    type Part2Result = usize;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        Ok(parsed
            .iter()
            .map(|vent_line| vent_line.vent_locations())
            .flatten()
            .counts()
            .into_iter()
            .filter(|(_loc, num_occurrences)| *num_occurrences > 1)
            .count())
    }
}
