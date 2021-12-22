#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use std::ops::RangeInclusive;
use std::str::FromStr;

use itertools::Itertools;
use regex::Regex;

pub struct Day22;

#[derive(Debug)]
struct Command {
    new_state: bool,
    region: Cuboid,
}

#[derive(Debug)]
struct Cuboid {
    ranges: Vec<RangeInclusive<i64>>,
}

impl Cuboid {
    fn contains(&self, loc: &Vec<i64>) -> bool {
        loc.iter()
            .zip(self.ranges.iter())
            .all(|(coordinate, range)| range.contains(coordinate))
    }

    fn locations(&self) -> impl Iterator<Item = Vec<i64>> + '_ {
        self.ranges.iter().cloned().multi_cartesian_product()
    }
}

impl FromStr for Command {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let captures = Regex::new(
            r"(?x)
             (?P<state>(on)|(off))
             \s
             x=
             (?P<xmin>-?\d+)
             \.\.
             (?P<xmax>-?\d+)
             ,
             y=
             (?P<ymin>-?\d+)
             \.\.
             (?P<ymax>-?\d+)
             ,
             z=
             (?P<zmin>-?\d+)
             \.\.
             (?P<zmax>-?\d+)
             ",
        )
        .unwrap()
        .captures(s)
        .ok_or(Error::Mismatch)?;

        let new_state = captures.name("state").unwrap().as_str() == "on";

        let ranges = ["xmin", "xmax", "ymin", "ymax", "zmin", "zmax"]
            .iter()
            .map(|name| captures.name(name).unwrap().as_str().parse::<i64>())
            .tuples()
            .map(|(a, b)| Ok((a?)..=(b?)))
            .collect::<Result<_, Error>>()?;
        let region = Cuboid { ranges };

        Ok(Self { new_state, region })
    }
}

impl Day22 {
    fn parse_commands(&self) -> Result<Vec<Command>, Error> {
        let puzzle_input = self.puzzle_input(PuzzleInput::Example(1))?;
        //let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let cuboids = puzzle_input
            .lines()
            .map(|line| line.parse())
            .collect::<Result<_, _>>()?;

        Ok(cuboids)
    }
}

impl Puzzle for Day22 {
    fn day(&self) -> i32 {
        22
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let commands = self.parse_commands()?;

        let region = Cuboid {
            ranges: vec![-50..=50, -50..=50, -50..=50],
        };

        let mut state: Vec<bool> = region.locations().map(|_| false).collect();

        commands.iter().for_each(|command| {
            region
                .locations()
                .map(|loc| command.region.contains(&loc))
                .zip(state.iter_mut())
                .filter_map(|(to_update, out)| to_update.then(|| out))
                .for_each(|out| {
                    *out = command.new_state;
                });
        });

        let result = state.iter().filter(|b| **b).count();
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result = ();
        Ok(Box::new(result))
    }
}
