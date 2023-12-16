use aoc_utils::prelude::*;

use std::{ops::RangeInclusive, str::FromStr};

#[derive(Clone, Copy)]
pub enum CommandType {
    TurnOn,
    Toggle,
    TurnOff,
}

pub struct Command {
    ty: CommandType,
    x_range: RangeInclusive<i64>,
    y_range: RangeInclusive<i64>,
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut iter = line.split_ascii_whitespace();
        let command_ty = iter.next().ok_or(Error::UnexpectedEndOfStream)?;

        let command_ty = match command_ty {
            "toggle" => Ok(CommandType::Toggle),
            "turn" => {
                let value = iter.next().ok_or(Error::UnexpectedEndOfStream)?;
                match value {
                    "on" => Ok(CommandType::TurnOn),
                    "off" => Ok(CommandType::TurnOff),
                    _ => Err(Error::InvalidString(value.to_string())),
                }
            }
            _ => Err(Error::InvalidString(command_ty.to_string())),
        }?;

        let (xmin, ymin) = iter
            .next()
            .ok_or(Error::UnexpectedEndOfStream)?
            .split(',')
            .map(|s| s.parse::<i64>())
            .tuples()
            .exactly_one_or_err()?;
        iter.next(); // skip the word "through"
        let (xmax, ymax) = iter
            .next()
            .ok_or(Error::UnexpectedEndOfStream)?
            .split(',')
            .map(|s| s.parse::<i64>())
            .tuples()
            .exactly_one_or_err()?;

        Ok(Command {
            ty: command_ty,
            x_range: xmin?..=xmax?,
            y_range: ymin?..=ymax?,
        })
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<Command>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        commands: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let mut state = GridMap::new_uniform(1000, 1000, false);
        commands
            .iter()
            .flat_map(|command| {
                let x_range = command.x_range.clone();
                let y_range = command.y_range.clone();
                x_range
                    .cartesian_product(y_range)
                    .map(move |pos| (pos, command.ty))
            })
            .for_each(|(pos, ty)| match ty {
                CommandType::TurnOn => {
                    state[pos] = true;
                }
                CommandType::Toggle => {
                    state[pos] = !state[pos];
                }
                CommandType::TurnOff => {
                    state[pos] = false;
                }
            });

        // let num_lights_on: i64 =
        //     state.iter().map(|light: &bool| *light as i64).sum::<i64>();
        let num_lights_on: i64 =
            state.iter().map(|&light| light as i64).sum::<i64>();

        Ok(num_lights_on)
    }

    fn part_2(
        commands: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let mut state = GridMap::new_uniform(1000, 1000, 0u64);
        commands
            .iter()
            .flat_map(|command| {
                let x_range = command.x_range.clone();
                let y_range = command.y_range.clone();
                x_range
                    .cartesian_product(y_range)
                    .map(move |pos| (pos, command.ty))
            })
            .for_each(|(pos, ty)| {
                let prev: u64 = state[pos];
                match ty {
                    CommandType::TurnOn => {
                        state[pos] = prev + 1;
                    }
                    CommandType::Toggle => {
                        state[pos] = prev + 2;
                    }
                    CommandType::TurnOff => {
                        state[pos] = prev.saturating_sub(1);
                    }
                }
            });

        let total_brightness: u64 =
            state.iter().copied().sum::<u64>();

        Ok(total_brightness)
    }
}
