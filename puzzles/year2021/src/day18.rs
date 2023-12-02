use aoc_utils::prelude::*;

use std::borrow::BorrowMut;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

#[derive(Clone)]
pub enum Snailfish {
    Value(i64),
    Pair(Box<Snailfish>, Box<Snailfish>),
}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    OpenBracket,
    CloseBracket,
    Value(i64),
    Comma,
}

#[derive(Debug)]
struct SnailfishView<'a> {
    snailfish: &'a Snailfish,
}

#[derive(Debug)]
struct SnailfishMutView<'a> {
    snailfish: &'a mut Snailfish,
    depth: u8,
}

struct TokenStream<'a> {
    char_iter: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> TokenStream<'a> {
    fn new(s: &'a str) -> Self {
        let char_iter = s.chars().peekable();
        Self { char_iter }
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let curr_char = self.char_iter.next()?;
        match curr_char {
            '[' => Some(Ok(Token::OpenBracket)),
            ']' => Some(Ok(Token::CloseBracket)),
            ',' => Some(Ok(Token::Comma)),
            '0'..='9' => {
                let first_digit = curr_char.to_digit(10).unwrap() as i64;
                let value = self
                    .char_iter
                    .peeking_take_while(|next_char| next_char.is_ascii_digit())
                    .fold(first_digit, |acc, c| {
                        let digit = c.to_digit(10).unwrap() as i64;
                        10 * acc + digit
                    });
                Some(Ok(Token::Value(value)))
            }
            _ => Some(Err(Error::UnknownChar(curr_char))),
        }
    }
}

impl<'a> TokenStream<'a> {
    fn check_next_token(&mut self, expected: Token) -> Result<(), Error> {
        let opt_next = self.next();
        if let Some(next) = opt_next {
            let next = next?;
            if next == expected {
                Ok(())
            } else {
                Err(Error::UnexpectedToken(format!(
                    "Expected {expected:?} but found {next:?}"
                )))
            }
        } else {
            Err(Error::UnexpectedEndOfStream)
        }
    }
}

impl FromStr for Snailfish {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let mut token_iter = TokenStream::new(s);
        Snailfish::from_token_stream(&mut token_iter)
    }
}

impl Debug for Snailfish {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Snailfish::Value(val) => write!(f, "{val}"),
            Snailfish::Pair(left, right) => {
                write!(f, "[{left:?}, {right:?}]")
            }
        }
    }
}

impl From<i64> for Snailfish {
    fn from(val: i64) -> Self {
        Snailfish::Value(val)
    }
}

impl<T1, T2> From<(T1, T2)> for Snailfish
where
    T1: Into<Snailfish>,
    T2: Into<Snailfish>,
{
    fn from((left, right): (T1, T2)) -> Self {
        let left = Box::new(left.into());
        let right = Box::new(right.into());
        Snailfish::Pair(left, right)
    }
}

impl Snailfish {
    fn from_token_stream(token_iter: &mut TokenStream) -> Result<Self, Error> {
        let token = token_iter.next().ok_or(Error::UnexpectedEndOfStream)??;

        match token {
            Token::Value(val) => Ok(val.into()),
            Token::OpenBracket => {
                let left = Self::from_token_stream(token_iter)?;
                token_iter.check_next_token(Token::Comma)?;
                let right = Self::from_token_stream(token_iter)?;
                token_iter.check_next_token(Token::CloseBracket)?;
                Ok((left, right).into())
            }
            _ => Err(Error::UnexpectedToken(format!("{token:?}"))),
        }
    }

    fn iter_mut<'a, P>(
        &'a mut self,
        mut predicate: P,
    ) -> impl Iterator<Item = SnailfishMutView> + 'a
    where
        P: 'a + FnMut(SnailfishView) -> bool,
    {
        let mut stack: Vec<SnailfishMutView<'a>> = vec![SnailfishMutView {
            snailfish: self,
            depth: 0,
        }];
        std::iter::from_fn(move || -> Option<SnailfishMutView> {
            loop {
                let mut_view = stack.pop()?;
                if predicate(SnailfishView {
                    snailfish: mut_view.snailfish,
                }) {
                    return Some(mut_view);
                }
                if let Snailfish::Pair(ref mut left, ref mut right) =
                    mut_view.snailfish
                {
                    stack.push(SnailfishMutView {
                        snailfish: right.borrow_mut(),
                        depth: mut_view.depth + 1,
                    });
                    stack.push(SnailfishMutView {
                        snailfish: left.borrow_mut(),
                        depth: mut_view.depth + 1,
                    });
                }
            }
        })
    }

    fn is_leaf_pair(&self) -> bool {
        use Snailfish::*;
        match self {
            Pair(left, right) => left.is_value() && right.is_value(),
            _ => false,
        }
    }

    fn is_value(&self) -> bool {
        self.as_value().is_some()
    }

    fn as_value(&self) -> Option<i64> {
        use Snailfish::*;
        match self {
            Value(val) => Some(*val),
            Pair(_, _) => None,
        }
    }

    fn as_mut_value(&mut self) -> Option<&mut i64> {
        use Snailfish::*;
        match self {
            Value(val) => Some(val),
            Pair(_, _) => None,
        }
    }

    fn magnitude(&self) -> i64 {
        use Snailfish::*;
        match self {
            Value(val) => *val,
            Pair(left, right) => 3 * left.magnitude() + 2 * right.magnitude(),
        }
    }

    fn after_reduce(&self) -> Self {
        let mut fish = self.clone();
        fish.reduce();
        fish
    }

    fn reduce(&mut self) {
        while self.reduce_step() {}
    }

    fn reduce_step(&mut self) -> bool {
        self.explode_step() || self.split_step()
    }

    fn explode_step(&mut self) -> bool {
        let mut leaves = self
            .iter_mut(|view| view.snailfish.as_value().is_some())
            .collect::<Vec<_>>();
        let append_steps = leaves
            .iter()
            .enumerate()
            .filter(|&(_i, leaf)| (leaf.depth > 4))
            .map(|(i, leaf)| (i, leaf.snailfish.as_value().unwrap()))
            .take(2)
            .enumerate()
            .filter_map(|(left_right, (i_explode, val))| {
                (i_explode + 2 * left_right)
                    .checked_sub(1)
                    .map(|i_target| (i_target, val))
            })
            .filter(|(i_target, _val)| *i_target < leaves.len())
            .collect::<Vec<_>>();

        let explode_performed = !append_steps.is_empty();

        append_steps.into_iter().for_each(|(i_target, val)| {
            *leaves[i_target].snailfish.as_mut_value().unwrap() += val;
        });

        self.iter_mut(|view| view.snailfish.is_leaf_pair())
            .find(|view| view.depth >= 4)
            .iter_mut()
            .for_each(|view| {
                *view.snailfish = 0.into();
            });

        explode_performed
    }

    fn split_step(&mut self) -> bool {
        let mut to_split = self
            .iter_mut(|view| view.snailfish.is_value())
            .find(|view| view.snailfish.as_value().unwrap() >= 10);

        to_split.iter_mut().for_each(|view| {
            let val = view.snailfish.as_value().unwrap();
            let left = val / 2;
            let right = val - left;
            *view.snailfish = (left, right).into()
        });

        to_split.is_some()
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 7;

    type ParsedInput = Vec<Snailfish>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(parsed
            .iter()
            .cloned()
            .reduce(|acc, fish| Snailfish::from((acc, fish)).after_reduce())
            .unwrap()
            .magnitude())
    }

    fn part_2(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(parsed
            .iter()
            .cloned()
            .permutations(2)
            .map(|permutation| {
                permutation
                    .into_iter()
                    .reduce(|acc, fish| {
                        Snailfish::from((acc, fish)).after_reduce()
                    })
                    .unwrap()
                    .magnitude()
            })
            .max()
            .unwrap())
    }
}
