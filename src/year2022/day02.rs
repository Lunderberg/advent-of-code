#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

pub struct ThisDay;

#[derive(PartialEq, Clone, Copy)]
enum Shape {
    Rock,
    Paper,
    Scissors,
}

enum Col2 {
    X,
    Y,
    Z,
}

impl Shape {
    fn score(&self) -> i32 {
        match self {
            Shape::Rock => 1,
            Shape::Paper => 2,
            Shape::Scissors => 3,
        }
    }

    fn to_cause(&self, outcome: RoundResult) -> Shape {
        use RoundResult::*;
        use Shape::*;
        match (outcome, self) {
            (Draw, _) => *self,
            (Win, Rock) => Paper,
            (Win, Paper) => Scissors,
            (Win, Scissors) => Rock,
            (Loss, Rock) => Scissors,
            (Loss, Paper) => Rock,
            (Loss, Scissors) => Paper,
        }
    }
}

#[derive(Copy, Clone)]
enum RoundResult {
    Win,
    Loss,
    Draw,
}

impl RoundResult {
    fn score(&self) -> i32 {
        match self {
            RoundResult::Win => 6,
            RoundResult::Draw => 3,
            RoundResult::Loss => 0,
        }
    }
}

struct Round {
    player: Shape,
    opponent: Shape,
}

impl Round {
    fn result(&self) -> RoundResult {
        use RoundResult::*;
        use Shape::*;
        match (&self.player, &self.opponent) {
            (a, b) if a == b => Draw,
            (Rock, Scissors) => Win,
            (Scissors, Paper) => Win,
            (Paper, Rock) => Win,
            _ => Loss,
        }
    }

    fn score(&self) -> i32 {
        self.player.score() + self.result().score()
    }
}

pub struct StrategyGuide {
    opponent: Shape,
    col2: Col2,
}

impl StrategyGuide {
    fn col2_as_rock_paper_scissors(&self) -> Round {
        let player = match self.col2 {
            Col2::X => Shape::Rock,
            Col2::Y => Shape::Paper,
            Col2::Z => Shape::Scissors,
        };
        Round {
            player,
            opponent: self.opponent,
        }
    }

    fn col2_as_win_loss_draw(&self) -> Round {
        let outcome = match self.col2 {
            Col2::X => RoundResult::Loss,
            Col2::Y => RoundResult::Draw,
            Col2::Z => RoundResult::Win,
        };

        let player = self.opponent.to_cause(outcome);

        Round {
            player,
            opponent: self.opponent,
        }
    }
}

impl std::str::FromStr for StrategyGuide {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Shape::*;

        let mut chars = s.chars();

        let opponent = match chars.next().ok_or(Error::NoneError)? {
            'A' => Ok(Rock),
            'B' => Ok(Paper),
            'C' => Ok(Scissors),
            c => Err(Error::UnknownChar(c)),
        }?;

        chars.next();

        let col2 = match chars.next().ok_or(Error::NoneError)? {
            'X' => Ok(Col2::X),
            'Y' => Ok(Col2::Y),
            'Z' => Ok(Col2::Z),
            c => Err(Error::UnknownChar(c)),
        }?;

        Ok(StrategyGuide { col2, opponent })
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 2;

    type ParsedInput = Vec<StrategyGuide>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    type Part1Result = i32;
    fn part_1(values: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        Ok(values
            .iter()
            .map(|guide| guide.col2_as_rock_paper_scissors())
            .map(|round| round.score())
            .sum())
    }

    type Part2Result = i32;
    fn part_2(values: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        Ok(values
            .iter()
            .map(|guide| guide.col2_as_win_loss_draw())
            .map(|round| round.score())
            .sum())
    }
}
