#![allow(unused_imports)]
use crate::utils::{extensions::*, Adjacency, GridMap};
use crate::{Error, Puzzle};

use itertools::Itertools;

pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 8;

    type ParsedInput = GridMap<u8>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines.collect())
    }

    type Part1Result = usize;
    fn part_1(gridmap: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        Ok(gridmap
            .iter()
            .filter(|(pos, height)| {
                Adjacency::Rook.offsets().any(|offset| {
                    gridmap.iter_ray(*pos, offset).skip(1).all(
                        |(_other_pos, &other_height)| **height > other_height,
                    )
                })
            })
            .count())
    }

    type Part2Result = usize;
    fn part_2(gridmap: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        Ok(gridmap
            .iter()
            .map(|(pos, height)| {
                Adjacency::Rook
                    .offsets()
                    .map(|offset| {
                        gridmap
                            .iter_ray(pos, offset)
                            .skip(1)
                            .take_while_inclusive(|(_, other_height)| {
                                *other_height < height
                            })
                            .count()
                    })
                    .product::<usize>()
            })
            .max()
            .unwrap())
    }
}
