#![allow(unused_imports)]
use crate::utils::extensions::*;
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

pub struct Day16;

#[derive(Debug)]
struct Packet {
    version: u8, // 3 bits
    type_id: u8, // 3 bits
    payload: PacketData,
}

#[derive(Debug)]
enum PacketData {
    Literal(Literal),
    Operator(Operator),
}

#[derive(Debug)]
struct Literal {
    value: u64,
}

#[derive(Debug)]
struct Operator {
    length: Length,
    subpackets: Vec<Packet>,
}

#[derive(Debug)]
enum Length {
    BitCount(u16),
    SubPacketCount(u16),
}

impl Packet {
    fn parse(
        bit_stream: &mut impl Iterator<Item = bool>,
    ) -> Result<Self, Error> {
        let version = bit_stream.by_ref().take(3).collect_bits();
        let type_id = bit_stream.by_ref().take(3).collect_bits();
        let payload = PacketData::parse(type_id, bit_stream)?;

        Ok(Self {
            version,
            type_id,
            payload,
        })
    }
}

impl PacketData {
    fn parse(
        type_id: u8,
        bit_stream: &mut impl Iterator<Item = bool>,
    ) -> Result<Self, Error> {
        Ok(match type_id {
            4 => Self::Literal(Literal::parse(bit_stream)?),
            _ => Self::Operator(Operator::parse(bit_stream)?),
        })
    }
}

impl Literal {
    fn parse(
        bit_stream: &mut impl Iterator<Item = bool>,
    ) -> Result<Self, Error> {
        let value = bit_stream
            .by_ref()
            .tuples()
            .take_while_inclusive(|(b, _, _, _, _)| *b)
            .flat_map(|(_, a, b, c, d)| {
                core::array::IntoIter::new([a, b, c, d])
            })
            .collect_bits();
        Ok(Self { value })
    }
}

impl Operator {
    fn parse(
        bit_stream: &mut impl Iterator<Item = bool>,
    ) -> Result<Self, Error> {
        let length = Length::parse(bit_stream)?;
        let subpackets = Self::parse_subpackets(&length, bit_stream)?;

        Ok(Self { length, subpackets })
    }

    fn parse_subpackets(
        length: &Length,
        bit_stream: &mut (impl Iterator<Item = bool> + Sized),
    ) -> Result<Vec<Packet>, Error> {
        match length {
            Length::BitCount(num_bits) => {
                let mut substream: Box<dyn Iterator<Item = bool>> =
                    Box::new(bit_stream.by_ref().take(*num_bits as usize));
                Self::parse_remaining_subpackets(&mut substream)
            }
            Length::SubPacketCount(num_packets) => (0..*num_packets)
                .map(|_| Packet::parse(bit_stream))
                .collect::<Result<_, _>>(),
        }
    }

    fn parse_remaining_subpackets(
        bit_stream: &mut impl Iterator<Item = bool>,
    ) -> Result<Vec<Packet>, Error> {
        std::iter::from_fn(|| {
            let res = Packet::parse(bit_stream);
            match res {
                Ok(packet) => Some(Ok(packet)),
                Err(error) => {
                    if let Error::UnexpectedEndOfStream = error {
                        None
                    } else {
                        Some(Err(error))
                    }
                }
            }
        })
        .collect()
    }
}

impl Length {
    fn parse(
        bit_stream: &mut impl Iterator<Item = bool>,
    ) -> Result<Self, Error> {
        let type_bit = bit_stream.next().ok_or(Error::UnexpectedEndOfStream)?;
        if type_bit {
            let count = bit_stream.by_ref().take(11).collect_bits();
            Ok(Length::SubPacketCount(count))
        } else {
            let count = bit_stream.by_ref().take(15).collect_bits();
            Ok(Length::BitCount(count))
        }
    }
}

impl Day16 {
    fn parse_inputs(&self) -> Result<Vec<bool>, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        // let puzzle_input = "D2FE28";
        //let puzzle_input = "38006F45291200";
        let puzzle_input = "EE00D40C823060";

        //let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        Ok(puzzle_input
            .chars()
            .map(|c| usize::from_str_radix(&c.to_string(), 16))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flat_map(|val| {
                (0..4).rev().map(move |bit| ((1 << bit) & val) != 0)
            })
            .collect())
    }
}

impl Puzzle for Day16 {
    fn day(&self) -> i32 {
        16
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let mut bit_stream = self.parse_inputs()?.into_iter();
        let result = Packet::parse(&mut bit_stream)?;
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        //let puzzle_input = self.puzzle_input(PuzzleInput::User)?;
        let result = ();
        Ok(Box::new(result))
    }
}
