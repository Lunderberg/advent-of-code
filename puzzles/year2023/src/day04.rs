use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
};

use aoc_utils::prelude::*;

#[derive(Debug)]
pub struct Card {
    id: u64,
    mine: Vec<u64>,
    winning: Vec<u64>,
}

impl FromStr for Card {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_ascii_whitespace();

        iter.next(); // Skip "Card"

        let id = iter
            .next()
            .ok_or(Error::UnexpectedEndOfStream)?
            .trim_end_matches(':')
            .parse()?;

        let mine = iter
            .by_ref()
            .take_while(|item| item != &"|")
            .map(|item| item.parse())
            .collect::<Result<Vec<_>, _>>()?;
        let winning = iter
            .map(|item| item.parse())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Card { id, mine, winning })
    }
}

impl Card {
    fn num_matches(&self) -> u64 {
        self.mine
            .iter()
            .cartesian_product(self.winning.iter())
            .filter(|(a, b)| a == b)
            .count() as u64
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<Card>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        cards: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(cards
            .iter()
            .map(|card| {
                let n = card.num_matches() as u32;
                if n == 0 {
                    0
                } else {
                    2_u64.pow(n - 1)
                }
            })
            .sum::<u64>())
    }

    fn part_2(
        cards: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let mut num_matches: VecDeque<_> = cards
            .iter()
            .map(|card| (card.id, card.num_matches()))
            .collect();

        let mut num_cards: HashMap<u64, u64> = HashMap::new();

        while let Some((id, num_new_cards)) = num_matches.pop_front() {
            let child_sum: Option<_> = (1..=num_new_cards)
                .map(|i| i + id)
                .map(|child_id| num_cards.get(&child_id))
                .sum::<Option<u64>>();
            if let Some(child_sum) = child_sum {
                num_cards.insert(id, child_sum + 1);
            } else {
                num_matches.push_back((id, num_new_cards));
            }
        }

        let total_cards = num_cards.into_values().sum::<u64>();

        Ok(total_cards)
    }
}
