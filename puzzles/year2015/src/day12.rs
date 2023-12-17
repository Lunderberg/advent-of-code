use std::{collections::HashMap, iter::Peekable};

use aoc_utils::prelude::*;

#[derive(Debug)]
pub enum Item {
    Number(i64),
    String(String),
    List(Vec<Box<Item>>),
    Dict(HashMap<String, Box<Item>>),
}

impl Item {
    fn try_collect(
        iter: &mut Peekable<impl Iterator<Item = char>>,
    ) -> Result<Self, Error> {
        match iter.peek().ok_or(Error::UnexpectedEndOfStream)? {
            '0'..='9' => Ok(Item::Number(Item::expect_int(iter))),

            '-' => {
                iter.next();
                if matches!(iter.peek(), Some('0'..='9')) {
                    Ok(Item::Number(-Item::expect_int(iter)))
                } else {
                    Err(Error::ParseError)
                }
            }

            '"' => {
                iter.next();
                let item = Item::String(
                    iter.peeking_take_while(|c| *c != '"').collect(),
                );
                match iter.next() {
                    Some('"') => Ok(item),
                    None => Err(Error::UnexpectedEndOfStream),
                    Some(c) => Err(Error::UnknownChar(c)),
                }
            }

            '[' => {
                iter.next();
                let mut items = Vec::new();
                loop {
                    items.push(Box::new(Item::try_collect(iter)?));
                    match iter.next() {
                        Some(',') => {}
                        Some(']') => {
                            return Ok(Item::List(items));
                        }
                        None => {
                            return Err(Error::UnexpectedEndOfStream);
                        }
                        Some(c) => {
                            return Err(Error::UnknownChar(c));
                        }
                    }
                }
            }

            '{' => {
                iter.next();
                let mut items = HashMap::new();
                loop {
                    let Item::String(key) = Item::try_collect(iter)? else {
                        return Err(Error::ParseError);
                    };
                    match iter.next() {
                        Some(':') => {}
                        _ => {
                            return Err(Error::ParseError);
                        }
                    }
                    let value = Item::try_collect(iter)?;
                    items.insert(key, Box::new(value));

                    match iter.next() {
                        Some(',') => {}
                        Some('}') => {
                            return Ok(Item::Dict(items));
                        }
                        None => {
                            return Err(Error::UnexpectedEndOfStream);
                        }
                        Some(c) => {
                            return Err(Error::UnknownChar(c));
                        }
                    }
                }
            }

            c => Err(Error::UnknownChar(*c)),
        }
    }

    fn expect_int(iter: &mut Peekable<impl Iterator<Item = char>>) -> i64 {
        iter.peeking_take_while(|c| matches!(c, '0'..='9'))
            .map(|c| {
                c.to_digit(10)
                    .expect("as_digit() must succeed due to previous filter")
                    as i64
            })
            .fold(0i64, |a, b| 10 * a + b)
    }

    fn iter_numbers(&self) -> Box<dyn Iterator<Item = i64> + '_> {
        match self {
            Item::Number(val) => Box::new(std::iter::once(*val)),
            Item::String(_) => Box::new(std::iter::empty()),
            Item::List(list) => {
                Box::new(list.iter().flat_map(|item| item.iter_numbers()))
            }
            Item::Dict(dict) => {
                Box::new(dict.iter().flat_map(|(_, item)| item.iter_numbers()))
            }
        }
    }

    fn iter_non_red_numbers(&self) -> Box<dyn Iterator<Item = i64> + '_> {
        match self {
            Item::Number(val) => Box::new(std::iter::once(*val)),
            Item::String(_) => Box::new(std::iter::empty()),
            Item::List(list) => Box::new(
                list.iter().flat_map(|item| item.iter_non_red_numbers()),
            ),
            Item::Dict(dict)
                if dict.iter().any(|(_, item)| match &**item {
                    Item::String(value) => value == "red",
                    _ => false,
                }) =>
            {
                Box::new(std::iter::empty())
            }
            Item::Dict(dict) => Box::new(
                dict.iter()
                    .flat_map(|(_, item)| item.iter_non_red_numbers()),
            ),
        }
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Item;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        // let lines = vec!["{\"a\":{\"b\":4},\"c\":-1}"].into_iter();
        // let lines = vec!["[1,{\"c\":\"red\",\"b\":2},3]"].into_iter();

        Item::try_collect(
            &mut lines
                .enumerate()
                .flat_map(|(i, line)| {
                    (i != 0).then(|| '\n').into_iter().chain(line.chars())
                })
                .peekable(),
        )
    }

    fn part_1(item: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        let value = item.iter_numbers().sum::<i64>();
        Ok(value)
    }

    fn part_2(item: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        let value = item.iter_non_red_numbers().sum::<i64>();
        Ok(value)
    }
}
