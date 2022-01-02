#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::Puzzle;

use itertools::Itertools;

#[derive(Debug)]
pub enum Command {
    Up(i64),
    Down(i64),
    Forward(i64),
}

impl std::str::FromStr for Command {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(' ')
            .tuples()
            .map(|(a, b)| {
                let val = b.parse::<i64>()?;
                match a {
                    "up" => Ok(Command::Up(val)),
                    "down" => Ok(Command::Down(val)),
                    "forward" => Ok(Command::Forward(val)),
                    _ => Err(Error::InvalidArg(a.into())),
                }
            })
            .exactly_one()?
    }
}

#[derive(Debug)]
struct SubmarineState {
    forward_pos: i64,
    depth: i64,
    aim: i64,
}

impl std::default::Default for SubmarineState {
    fn default() -> Self {
        Self {
            forward_pos: 0,
            depth: 0,
            aim: 0,
        }
    }
}

fn final_position_part1(commands: &Vec<Command>) -> SubmarineState {
    let mut pos = SubmarineState::default();
    commands.iter().for_each(|c| match c {
        Command::Up(val) => {
            pos.depth -= val;
        }
        Command::Down(val) => {
            pos.depth += val;
        }
        Command::Forward(val) => {
            pos.forward_pos += val;
        }
    });
    pos
}

fn final_position_part2(commands: &Vec<Command>) -> SubmarineState {
    let mut pos = SubmarineState::default();
    commands.iter().for_each(|c| match c {
        Command::Up(val) => {
            pos.aim -= val;
        }
        Command::Down(val) => {
            pos.aim += val;
        }
        Command::Forward(val) => {
            pos.depth += val * pos.aim;
            pos.forward_pos += val;
        }
    });
    pos
}

pub struct Day02;

impl Puzzle for Day02 {
    const DAY: u8 = 2;
    const IMPLEMENTED: bool = true;
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<Command>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse::<Command>()).collect()
    }

    type Part1Result = i64;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        let pos = final_position_part1(&parsed);
        Ok(pos.depth * pos.forward_pos)
    }

    type Part2Result = i64;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        let pos = final_position_part2(&parsed);
        Ok(pos.depth * pos.forward_pos)
    }
}
