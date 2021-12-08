#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

#[derive(Debug)]
enum Command {
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
    fn day(&self) -> i32 {
        2
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let commands = puzzle_input
            .lines()
            .map(|line| line.parse::<Command>())
            .collect::<Result<Vec<_>, _>>()?;
        let pos = final_position_part1(&commands);

        let result = pos.depth * pos.forward_pos;
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;
        let commands = puzzle_input
            .lines()
            .map(|line| line.parse::<Command>())
            .collect::<Result<Vec<_>, _>>()?;
        let pos = final_position_part2(&commands);

        let result = pos.depth * pos.forward_pos;
        Ok(Box::new(result))
    }
}
