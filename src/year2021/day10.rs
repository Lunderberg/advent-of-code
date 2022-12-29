#![allow(unused_imports)]
use crate::{Error, Puzzle};

use std::fmt::{Debug, Formatter};

use itertools::Itertools;

pub struct ThisDay;

pub struct ProgramLine {
    tokens: Vec<Token>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Token {
    RoundOpen,
    RoundClose,
    SquareOpen,
    SquareClose,
    SquiggleOpen,
    SquiggleClose,
    PointyOpen,
    PointyClose,
}

enum ParseResult {
    OpenDelimiters(Vec<Token>),
    IllegalCharacter(Token),
}

impl Debug for ProgramLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "\"")?;
        self.tokens
            .iter()
            .try_for_each(|token| write!(f, "{token:?}"))?;
        write!(f, "\"")?;

        Ok(())
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Token::*;
        write!(
            f,
            "{}",
            match self {
                RoundOpen => "(",
                RoundClose => ")",
                SquareOpen => "[",
                SquareClose => "]",
                SquiggleOpen => "{",
                SquiggleClose => "}",
                PointyOpen => "<",
                PointyClose => ">",
            }
        )
    }
}

impl std::str::FromStr for Token {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        use Token::*;
        match s {
            "(" => Ok(RoundOpen),
            ")" => Ok(RoundClose),
            "[" => Ok(SquareOpen),
            "]" => Ok(SquareClose),
            "{" => Ok(SquiggleOpen),
            "}" => Ok(SquiggleClose),
            "<" => Ok(PointyOpen),
            ">" => Ok(PointyClose),
            _ => Err(Error::InvalidString(s.to_string())),
        }
    }
}

impl std::str::FromStr for ProgramLine {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let tokens = s
            .chars()
            .map(|c| c.to_string().parse())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { tokens })
    }
}

impl Token {
    fn closing_delimiter(&self) -> Result<Self, Error> {
        use Token::*;
        match self {
            RoundOpen => Ok(RoundClose),
            SquareOpen => Ok(SquareClose),
            SquiggleOpen => Ok(SquiggleClose),
            PointyOpen => Ok(PointyClose),
            _ => Err(Error::NotOpeningDelimiter),
        }
    }

    fn syntax_points(&self) -> u64 {
        use Token::*;
        match self {
            RoundClose => 3,
            SquareClose => 57,
            SquiggleClose => 1197,
            PointyClose => 25137,
            _ => 0,
        }
    }

    fn autocomplete_points(&self) -> u64 {
        use Token::*;
        match self {
            RoundClose => 1,
            SquareClose => 2,
            SquiggleClose => 3,
            PointyClose => 4,
            _ => 0,
        }
    }
}

impl ProgramLine {
    fn parse_brackets(&self) -> ParseResult {
        let mut stack = Vec::new();
        self.tokens
            .iter()
            .scan(&mut stack, |stack, &token| -> Option<Option<Token>> {
                let is_opening = token.closing_delimiter().is_ok();
                if is_opening {
                    stack.push(token);
                    Some(None)
                } else {
                    let opening = stack.pop()?;
                    if opening.closing_delimiter().unwrap() == token {
                        Some(None)
                    } else {
                        Some(Some(token))
                    }
                }
            })
            .flatten()
            .next()
            .map_or_else(
                || ParseResult::OpenDelimiters(stack),
                ParseResult::IllegalCharacter,
            )
    }
}

impl Puzzle for ThisDay {
    const YEAR: u32 = 2021;
    const DAY: u8 = 10;
    const EXAMPLE_NUM: u8 = 1;

    type ParsedInput = Vec<ProgramLine>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|s| s.parse()).collect::<Result<Vec<_>, _>>()
    }

    type Part1Result = u64;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        Ok(parsed
            .iter()
            .filter_map(|line| {
                if let ParseResult::IllegalCharacter(token) =
                    line.parse_brackets()
                {
                    Some(token)
                } else {
                    None
                }
            })
            .map(|token| token.syntax_points())
            .sum::<u64>())
    }

    type Part2Result = u64;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        let points = parsed
            .iter()
            .filter_map(|line| {
                if let ParseResult::OpenDelimiters(stack) =
                    line.parse_brackets()
                {
                    Some(stack)
                } else {
                    None
                }
            })
            .map(|stack| {
                stack
                    .iter()
                    .rev()
                    .map(|token| {
                        token.closing_delimiter().unwrap().autocomplete_points()
                    })
                    .fold(0, |acc, points| 5 * acc + points)
            })
            .sorted()
            .collect::<Vec<_>>();
        Ok(points[points.len() / 2])
    }
}
