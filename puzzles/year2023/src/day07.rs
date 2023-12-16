use std::{fmt::Display, str::FromStr};

use aoc_utils::prelude::*;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
struct Card(u8);

#[derive(Debug, PartialEq, Eq)]
pub struct Hand([Card; 5]);

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
enum Strength {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfKind,
    FullHouse,
    FourOfKind,
    FiveOfKind,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self.0 {
            14 => 'A',
            13 => 'K',
            12 => 'Q',
            11 => 'J',
            10 => 'T',
            1 => '*',
            digit => char::from_digit(digit as u32, 10)
                .expect("Invalid value stored in Card"),
        };
        write!(f, "{c}")
    }
}

impl TryFrom<char> for Card {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        let value = match c {
            'A' => 14,
            'K' => 13,
            'Q' => 12,
            'J' => 11,
            'T' => 10,
            '1'..='9' => c.to_digit(10).unwrap() as u8,
            _ => return Err(Error::UnknownChar(c)),
        };
        Ok(Self(value))
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_for_each(|card| write!(f, "{card}"))
    }
}

impl FromStr for Hand {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.chars()
                .map(|c| -> Result<Card, _> { c.try_into() })
                .collect::<Result<Vec<Card>, _>>()?
                .try_into()
                .map_err(|_| Error::WrongIteratorSize)?,
        ))
    }
}

impl Card {
    fn with_joker(&self) -> Card {
        Card(match self.0 {
            11 => 1,
            d => d,
        })
    }

    fn is_joker(&self) -> bool {
        self.0 == 1
    }
}

impl Hand {
    fn with_joker(&self) -> Hand {
        Hand(self.0.map(|card| card.with_joker()))
    }

    fn strength(&self) -> Strength {
        let counts: Vec<_> = self
            .0
            .iter()
            .filter(|c| !c.is_joker())
            .counts()
            .into_values()
            .sorted()
            .rev()
            .collect();
        let num_jokers = self.0.iter().filter(|c| c.is_joker()).count();

        let a = counts.first().cloned().unwrap_or(0) + num_jokers;
        let b = counts.get(1).cloned().unwrap_or(0);

        assert_eq!(counts.iter().sum::<usize>() + num_jokers, 5);
        assert!(a + b <= 5);

        let output = match (a, b) {
            (5, 0) => Strength::FiveOfKind,
            (4, 1) => Strength::FourOfKind,
            (3, 2) => Strength::FullHouse,
            (3, 1) => Strength::ThreeOfKind,
            (2, 2) => Strength::TwoPair,
            (2, 1) => Strength::OnePair,
            (1, 1) => Strength::HighCard,
            _ => panic!("Invalid result for {self}"),
        };

        assert!(num_jokers == 0 || output != Strength::HighCard);

        output
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<(Hand, u64)>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines
            .map(|line| -> Result<_, Error> {
                let (hand, bid) = line
                    .split_ascii_whitespace()
                    .collect_tuple()
                    .ok_or(Error::WrongIteratorSize)?;

                Ok((hand.parse()?, bid.parse()?))
            })
            .collect()
    }

    fn part_1(
        hands: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = hands
            .iter()
            .sorted_by_key(|(hand, _)| (hand.strength(), hand.0))
            .enumerate()
            .map(|(i, (_, bid))| {
                let i = i as u64;
                (i + 1) * bid
            })
            .sum::<u64>();
        Ok(value)
    }

    fn part_2(
        hands: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = hands
            .iter()
            .map(|(hand, bid)| (hand.with_joker(), bid))
            .sorted_by_key(|(hand, _)| (hand.strength(), hand.0))
            .enumerate()
            .map(|(i, (_, bid))| {
                let i = (i + 1) as u64;
                i * bid
            })
            .sum::<u64>();
        Ok(value)
    }
}
