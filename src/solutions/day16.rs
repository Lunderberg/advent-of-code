#![allow(unused_imports)]
use crate::utils::extensions::*;
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

pub struct Day16;

#[derive(Debug)]
struct Packet {
    version: u8, // 3 bits
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
    op: OperatorType,
    subpackets: Vec<Packet>,
}

#[derive(Debug)]
enum OperatorType {
    Sum,
    Product,
    Minimum,
    Maximum,
    GreaterThan,
    LessThan,
    EqualTo,
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
        let payload = PacketData::parse(bit_stream)?;

        Ok(Self { version, payload })
    }

    fn sum_version_nums(&self) -> u64 {
        let subpacket_sum = if let PacketData::Operator(op) = &self.payload {
            op.subpackets
                .iter()
                .map(|packet| packet.sum_version_nums())
                .sum::<u64>()
        } else {
            0
        };
        subpacket_sum + (self.version as u64)
    }

    fn eval(&self) -> Result<u64, Error> {
        self.payload.eval()
    }
}

impl PacketData {
    fn parse(
        bit_stream: &mut impl Iterator<Item = bool>,
    ) -> Result<Self, Error> {
        let type_id = bit_stream.by_ref().take(3).collect_bits();

        Ok(match type_id {
            4 => Self::Literal(Literal::parse(bit_stream)?),
            _ => Self::Operator(Operator::parse(type_id, bit_stream)?),
        })
    }

    fn eval(&self) -> Result<u64, Error> {
        match self {
            Self::Literal(literal) => Ok(literal.value),
            Self::Operator(op) => op.eval(),
        }
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
        type_id: u8,
        bit_stream: &mut impl Iterator<Item = bool>,
    ) -> Result<Self, Error> {
        let length = Length::parse(bit_stream)?;
        let subpackets = Self::parse_subpackets(&length, bit_stream)?;

        use OperatorType::*;
        let op = match type_id {
            0 => Sum,
            1 => Product,
            2 => Minimum,
            3 => Maximum,
            5 => GreaterThan,
            6 => LessThan,
            7 => EqualTo,
            _ => panic!("Unexpected type ID: {}", type_id),
        };

        Ok(Self { subpackets, op })
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

    fn eval(&self) -> Result<u64, Error> {
        let mut vals = self.subpackets.iter().map(|p| p.eval());

        fn binary_op(
            vals: impl Iterator<Item = Result<u64, Error>>,
        ) -> Result<(u64, u64), Error> {
            let (a, b) =
                vals.collect_tuple().ok_or(Error::IllegalNumberOfOperands)?;
            Ok((a?, b?))
        }

        use OperatorType::*;
        match self.op {
            Sum => vals.fold_ok(0, |acc, val| acc + val),
            Product => vals.fold_ok(1, |acc, val| acc * val),
            Minimum => vals
                .fold_ok(None, |acc, val| {
                    Some(acc.unwrap_or(u64::MAX).min(val))
                })
                .and_then(|opt_min| {
                    opt_min.ok_or(Error::IllegalNumberOfOperands)
                }),
            Maximum => vals
                .fold_ok(None, |acc, val| {
                    Some(acc.unwrap_or(u64::MIN).max(val))
                })
                .and_then(|opt_max| {
                    opt_max.ok_or(Error::IllegalNumberOfOperands)
                }),
            GreaterThan => binary_op(vals).map(|(a, b)| (a > b) as u64),
            LessThan => binary_op(vals).map(|(a, b)| (a < b) as u64),
            EqualTo => binary_op(vals).map(|(a, b)| (a == b) as u64),
        }
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

impl Puzzle for Day16 {
    const DAY: u8 = 16;
    const IMPLEMENTED: bool = true;
    const EXAMPLE_NUM: u8 = 1;

    type ParsedInput = Vec<bool>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines
            .flat_map(|line| line.chars())
            .map(|c| usize::from_str_radix(&c.to_string(), 16))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flat_map(|val| {
                (0..4).rev().map(move |bit| ((1 << bit) & val) != 0)
            })
            .collect())
    }

    type Part1Result = u64;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        Ok(Packet::parse(&mut parsed.iter().copied())?.sum_version_nums())
    }

    type Part2Result = u64;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        Packet::parse(&mut parsed.iter().copied())?.eval()
    }
}
