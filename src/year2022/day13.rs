#![allow(unused_imports)]
use crate::{Error, Puzzle};

use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::iter::Peekable;

use itertools::{EitherOrBoth, Itertools};

#[derive(Clone)]
pub enum Packet {
    Int(i64),
    List(Vec<Packet>),
}

#[derive(PartialEq, Eq)]
enum Token {
    Left,
    Right,
    Comma,
    Int(i64),
}

struct Tokenizer<I: Iterator<Item = char>> {
    iter: Peekable<I>,
}

struct Parser<I: Iterator<Item = Result<Token, Error>>> {
    iter: Peekable<I>,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Token::Left => write!(f, "["),
            Token::Right => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Int(val) => write!(f, "{val}"),
        }
    }
}

impl<I: Iterator<Item = char>> Tokenizer<I> {
    fn new(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for Tokenizer<I> {
    type Item = Result<Token, Error>;
    fn next(&mut self) -> Option<Result<Token, Error>> {
        self.iter.next().map(|c| match c {
            '[' => Ok(Token::Left),
            ']' => Ok(Token::Right),
            ',' => Ok(Token::Comma),
            '0'..='9' => {
                let mut value: i64 = ((c as u32) - ('0' as u32)) as i64;
                while matches!(self.iter.peek(), Some('0'..='9')) {
                    let digit = ((self.iter.next().unwrap() as u32)
                        - ('0' as u32)) as i64;
                    value = 10 * value + digit;
                }
                Ok(Token::Int(value))
            }
            _ => Err(Error::UnknownChar(c)),
        })
    }
}

impl<I: Iterator<Item = Result<Token, Error>>> Parser<I> {
    fn new(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
        }
    }

    fn expect_next(&mut self) -> Result<Token, Error> {
        self.iter.next().ok_or(Error::UnexpectedEndOfStream)?
    }

    fn expect_peek(&mut self) -> Result<&Token, Error> {
        let peek: Option<&Result<Token, Error>> = self.iter.peek();
        let res_ref: &Result<Token, Error> =
            peek.ok_or(Error::UnexpectedEndOfStream)?;
        let res: Result<&Token, &Error> = res_ref.as_ref();
        let res_raisable: Result<&Token, Error> =
            res.map_err(|err| match err {
                Error::UnknownChar(c) => Error::UnknownChar(*c),
                _ => panic!("Probably should implement a Clone somewhere"),
            });
        res_raisable
    }

    fn parse_packet(&mut self) -> Result<Packet, Error> {
        match self.expect_peek()? {
            Token::Left => self.expect_list(),
            Token::Int(_) => self.expect_int(),
            token => Err(Error::UnexpectedToken(format!(
                "Expected [ or int, but found {token}"
            ))),
        }
    }

    fn expect_int(&mut self) -> Result<Packet, Error> {
        match self.expect_next()? {
            Token::Int(val) => Ok(Packet::Int(val)),
            token => Err(Error::UnexpectedToken(format!(
                "Expected int, but found {token}"
            ))),
        }
    }

    fn expect_list(&mut self) -> Result<Packet, Error> {
        self.expect_token(Token::Left)?;
        let mut packets = Vec::new();
        match self.expect_peek()? {
            Token::Left | Token::Int(_) => {
                packets.push(self.parse_packet()?);
            }
            _ => (),
        }
        loop {
            match self.expect_next()? {
                Token::Right => {
                    break;
                }
                Token::Comma => (),
                token => Err(Error::UnexpectedToken(format!(
                    "Expected ] or comma, but found {token}"
                )))?,
            }

            packets.push(self.parse_packet()?);
        }
        Ok(Packet::List(packets))
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), Error> {
        let token = self.expect_next()?;
        if token == expected {
            Ok(())
        } else {
            Err(Error::UnexpectedToken(format!(
                "Expected {expected} but found {token}"
            )))
        }
    }

    fn expect_eof(&mut self) -> Result<(), Error> {
        match self.iter.next() {
            None => Ok(()),
            Some(Ok(token)) => Err(Error::UnexpectedToken(format!(
                "Expected EOF, but found {token}"
            ))),
            Some(Err(err)) => Err(err),
        }
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Packet::Int(val) => write!(f, "{val}"),
            Packet::List(vec) => {
                write!(f, "[")?;
                vec.iter().enumerate().try_for_each(|(i, subpacket)| {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{subpacket}")
                })?;
                write!(f, "]")?;
                Ok(())
            }
        }
    }
}

impl std::str::FromStr for Packet {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let tokens = Tokenizer::new(line.chars());
        let mut parser = Parser::new(tokens);
        let packet = parser.parse_packet()?;
        parser.expect_eof()?;
        Ok(packet)
    }
}

impl PartialEq for Packet {
    fn eq(&self, rhs: &Packet) -> bool {
        match (self, rhs) {
            (Packet::Int(a), Packet::Int(b)) => a == b,
            (Packet::Int(_), Packet::List(b)) => b.len() == 1 && self == &b[0],
            (Packet::List(a), Packet::Int(_)) => a.len() == 1 && &a[0] == rhs,
            (Packet::List(a), Packet::List(b)) => {
                a.len() == b.len()
                    && a.iter().zip(b.iter()).all(|(ai, bi)| ai == bi)
            }
        }
    }
}

impl Eq for Packet {}

impl PartialOrd for Packet {
    fn partial_cmp(&self, rhs: &Packet) -> Option<Ordering> {
        match (self, rhs) {
            (Packet::Int(a), Packet::Int(b)) => a.partial_cmp(b),
            (Packet::Int(_), Packet::List(b)) => {
                if b.is_empty() {
                    Some(Ordering::Greater)
                } else if b.len() == 1 {
                    self.partial_cmp(&b[0])
                } else if matches!(self.partial_cmp(&b[0])?, Ordering::Greater)
                {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            (Packet::List(_), Packet::Int(_)) => {
                rhs.partial_cmp(self).map(|cmp| cmp.reverse())
            }
            (Packet::List(a), Packet::List(b)) => Some(
                a.iter()
                    .zip_longest(b.iter())
                    .filter_map(|cmp| match cmp {
                        EitherOrBoth::Right(_) => Some(Ordering::Less),
                        EitherOrBoth::Left(_) => Some(Ordering::Greater),
                        EitherOrBoth::Both(ai, bi) => ai.partial_cmp(bi),
                    })
                    .find(|ord| ord != &Ordering::Equal)
                    .unwrap_or(Ordering::Equal),
            ),
        }
    }
}

impl Ord for Packet {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 13;

    type ParsedInput = Vec<Packet>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines
            .filter(|line| !line.is_empty())
            .map(|line| line.parse::<Packet>())
            .collect()
    }

    type Part1Result = usize;
    fn part_1(values: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        Ok(values
            .iter()
            .tuples()
            .enumerate()
            .filter_map(|(i, (a, b))| (a < b).then_some(i + 1))
            .sum())
    }

    type Part2Result = usize;
    fn part_2(values: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        let divider_a: Packet = "[[2]]".parse()?;
        let divider_b: Packet = "[[6]]".parse()?;
        let (loc_a, loc_b) = values
            .iter()
            .chain(vec![&divider_a, &divider_b].into_iter())
            .sorted()
            .enumerate()
            .filter_map(|(i, packet)| {
                (packet == &divider_a || packet == &divider_b).then_some(i + 1)
            })
            .tuples()
            .exactly_one()?;

        Ok(loc_a * loc_b)
    }
}
