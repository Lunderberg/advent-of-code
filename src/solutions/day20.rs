#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use crate::utils::{Adjacency, GridMap};

use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use itertools::Itertools;

pub struct Day20;

#[derive(Debug)]
struct Image {
    enhancement: Vec<Pixel>,
    grid: GridMap<Pixel>,
    pad_value: Pixel,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Pixel {
    Light,
    Dark,
}

impl Display for Pixel {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let c = match self {
            Pixel::Light => '#',
            Pixel::Dark => '.',
        };
        write!(f, "{}", c)
    }
}

impl Display for Image {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let enhancement =
            self.enhancement.iter().map(|p| format!("{}", p)).join("");
        write!(
            f,
            "{}\n\nInfinite: {:?}\n{}",
            enhancement, self.pad_value, self.grid,
        )
    }
}

impl FromStr for Pixel {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let c = s.chars().exactly_one()?;
        match c {
            '.' => Ok(Pixel::Dark),
            '#' => Ok(Pixel::Light),
            _ => Err(Error::UnknownChar(c)),
        }
    }
}

impl Image {
    fn enhance(&self) -> Self {
        let enhancement = self.enhancement.clone();

        let x_pad = 1;
        let y_pad = 1;

        let grid = (0..self.grid.x_size + 2 * (x_pad as usize))
            .flat_map(|x| {
                (0..self.grid.y_size + 2 * (y_pad as usize))
                    .map(move |y| (x, y))
            })
            .map(|(x, y)| {
                let orig_pos = ((x as i64) - x_pad, (y as i64) - y_pad);

                let index = self
                    .grid
                    .adjacent_values_default(
                        orig_pos,
                        Adjacency::Region3x3,
                        self.pad_value,
                    )
                    .map(|p| match p {
                        Pixel::Light => 1,
                        Pixel::Dark => 0,
                    })
                    .fold(0, |acc, val| 2 * acc + val);

                let value = self.enhancement[index];

                (x, y, value)
            })
            .collect();

        let pad_value = self.enhancement[match self.pad_value {
            Pixel::Dark => 0,
            Pixel::Light => 2_usize.pow(9) - 1,
        }];

        Self {
            enhancement,
            grid,
            pad_value,
        }
    }

    fn num_pixels(&self) -> usize {
        self.grid
            .iter()
            .filter(|(_pos, p)| **p == Pixel::Light)
            .count()
    }
}

impl Day20 {
    fn parse_image(&self) -> Result<Image, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let mut line_iter = puzzle_input.lines();
        let enhancement = line_iter
            .by_ref()
            .take_while(|line| line.len() > 0)
            .flat_map(|line| line.chars())
            .map(|c| c.to_string().parse::<Pixel>())
            .collect::<Result<_, _>>()?;

        let grid = line_iter.collect();

        let pad_value = Pixel::Dark;

        Ok(Image {
            enhancement,
            grid,
            pad_value,
        })
    }
}

impl Puzzle for Day20 {
    fn day(&self) -> i32 {
        20
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let image = self.parse_image()?;
        let result = (0..2).fold(image, |acc, _i| acc.enhance()).num_pixels();
        Ok(Box::new(result))
    }

    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let image = self.parse_image()?;
        let result = (0..50).fold(image, |acc, _i| acc.enhance()).num_pixels();
        Ok(Box::new(result))
    }
}
