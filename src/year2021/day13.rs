#![allow(unused_imports)]
use crate::{Error, Puzzle};

use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

pub struct ThisDay;

#[derive(Debug, Clone)]
pub struct Transparency {
    dots: HashSet<(i64, i64)>,
}

#[derive(Debug, Clone, Copy)]
pub enum FoldInstruction {
    X(i64),
    Y(i64),
}

impl std::str::FromStr for FoldInstruction {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split('=')
            .tuples()
            .map(|(a, b)| {
                let fold_pos = b.parse::<i64>()?;
                match a {
                    "fold along x" => Ok(FoldInstruction::X(fold_pos)),
                    "fold along y" => Ok(FoldInstruction::Y(fold_pos)),
                    _ => Err(Error::InvalidString(s.to_string())),
                }
            })
            .exactly_one()?
    }
}

impl Display for Transparency {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let x_max = *self.dots.iter().map(|(x, _y)| x).max().unwrap();
        let y_max = *self.dots.iter().map(|(_x, y)| y).max().unwrap();

        (0..=y_max)
            .cartesian_product(0..=x_max)
            .try_for_each(|(y, x)| {
                let c = if self.dots.contains(&(x, y)) {
                    '#'
                } else {
                    '.'
                };
                write!(f, "{}", c)?;
                if x == x_max {
                    writeln!(f)?;
                }
                Ok(())
            })
    }
}

impl Transparency {
    fn after_fold(&self, fold: FoldInstruction) -> Result<Self, Error> {
        let dots = self
            .dots
            .iter()
            .map(|(x, y)| match fold {
                FoldInstruction::X(fold) => {
                    Self::fold_coordinate(*x, fold).map(|x_new| (x_new, *y))
                }
                FoldInstruction::Y(fold) => {
                    Self::fold_coordinate(*y, fold).map(|y_new| (*x, y_new))
                }
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { dots })
    }

    fn fold_coordinate(coordinate: i64, fold_line: i64) -> Result<i64, Error> {
        use std::cmp::Ordering::*;
        match coordinate.cmp(&fold_line) {
            Less => Ok(coordinate),
            Greater => Ok(fold_line - (coordinate - fold_line)),
            Equal => Err(Error::DotOnFoldLine),
        }
    }
}

impl Puzzle for ThisDay {
    const YEAR: u32 = 2021;
    const DAY: u8 = 13;
    const EXAMPLE_NUM: u8 = 1;

    type ParsedInput = (Transparency, Vec<FoldInstruction>);
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let dots = lines
            .by_ref()
            .take_while(|line| !line.is_empty())
            .map(|line| -> Result<_, Error> {
                line.split(',')
                    .map(|s| s.parse::<i64>())
                    .tuples()
                    .map(|(a, b)| -> Result<_, Error> { Ok((a?, b?)) })
                    .exactly_one()?
            })
            .collect::<Result<_, _>>()?;

        let instructions = lines
            .map(|line| line.parse::<FoldInstruction>())
            .collect::<Result<Vec<_>, _>>()?;

        Ok((Transparency { dots }, instructions))
    }

    type Part1Result = usize;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        let (paper, folds) = parsed;
        let fold = folds.iter().cloned().next().ok_or(Error::NoneError)?;

        let folded = paper.after_fold(fold)?;

        Ok(folded.dots.len())
    }

    type Part2Result = String;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        let (paper, folds) = parsed;
        let mut paper = paper.clone();

        folds.iter().try_for_each(|&fold| -> Result<_, Error> {
            paper = paper.after_fold(fold)?;
            Ok(())
        })?;

        Ok(format!("{}", paper))
    }
}
