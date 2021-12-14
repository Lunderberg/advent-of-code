#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

pub struct Day13;

#[derive(Debug, Clone)]
struct Transparency {
    dots: HashSet<(i64, i64)>,
}

#[derive(Debug, Clone)]
enum FoldInstruction {
    X(i64),
    Y(i64),
}

impl std::str::FromStr for FoldInstruction {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split("=")
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
                    write!(f, "\n")?;
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
        if coordinate < fold_line {
            Ok(coordinate)
        } else if coordinate > fold_line {
            Ok(fold_line - (coordinate - fold_line))
        } else {
            Err(Error::DotOnFoldLine)
        }
    }
}

impl Day13 {
    fn parse_input(
        &self,
    ) -> Result<(Transparency, Vec<FoldInstruction>), Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(1))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let mut line_iter = puzzle_input.lines();

        let dots = line_iter
            .by_ref()
            .take_while(|line| line.len() > 0)
            .map(|line| -> Result<_, Error> {
                Ok(line
                    .split(',')
                    .map(|s| s.parse::<i64>())
                    .tuples()
                    .map(|(a, b)| -> Result<_, Error> { Ok((a?, b?)) })
                    .exactly_one()??)
            })
            .collect::<Result<_, _>>()?;

        let instructions = line_iter
            .map(|line| line.parse::<FoldInstruction>())
            .collect::<Result<Vec<_>, _>>()?;

        Ok((Transparency { dots }, instructions))
    }
}

impl Puzzle for Day13 {
    fn day(&self) -> i32 {
        13
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let (paper, folds) = self.parse_input()?;
        let fold = folds.iter().cloned().next().ok_or(Error::NoneError)?;

        let folded = paper.after_fold(fold)?;

        let result = folded.dots.len();
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let (mut paper, folds) = self.parse_input()?;

        folds.into_iter().try_for_each(|fold| -> Result<_, Error> {
            paper = paper.after_fold(fold)?;
            Ok(())
        })?;

        println!("{}", paper);

        let result = ();
        Ok(Box::new(result))
    }
}
