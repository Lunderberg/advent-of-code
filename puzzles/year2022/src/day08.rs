use aoc_utils::prelude::*;

use crate::utils::{Adjacency, GridMap};

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = GridMap<u8>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines.collect())
    }

    fn part_1(
        gridmap: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(gridmap
            .iter()
            .filter(|&(pos, &height)| {
                Adjacency::Rook.offsets().any(|offset| {
                    gridmap.iter_ray(pos, offset).skip(1).all(
                        |(_other_pos, &other_height)| height > other_height,
                    )
                })
            })
            .count())
    }

    fn part_2(
        gridmap: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(gridmap
            .iter_pos()
            .map(|(pos, &height)| {
                Adjacency::Rook
                    .offsets()
                    .map(|offset| {
                        gridmap
                            .iter_ray(pos, offset)
                            .skip(1)
                            .take_while_inclusive(|(_, &other_height)| {
                                other_height < height
                            })
                            .count()
                    })
                    .product::<usize>()
            })
            .max()
            .unwrap())
    }
}
