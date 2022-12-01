#![allow(unused_imports)]

use std::collections::VecDeque;

use crate::{Error, Puzzle};

use itertools::Itertools;

pub struct Day04;

#[derive(Debug, Clone)]
struct BingoTile {
    number: u32,
    called: bool,
}

#[derive(Debug, Clone)]
struct BingoBoard {
    tiles: Vec<BingoTile>,
}

#[derive(Debug, Clone)]
pub struct BingoGame {
    numbers_called: Vec<u32>,
    boards: Vec<BingoBoard>,
}

enum WhichBoard {
    First,
    Last,
}

impl std::fmt::Display for BingoTile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let highlight = if self.called { "*" } else { " " };
        write!(f, "{}{:#02}{}", highlight, self.number, highlight)
    }
}

impl BingoBoard {
    fn straight_lines() -> impl Iterator<Item = impl Iterator<Item = usize>> {
        let rows = (0..5).map(|rownum| (rownum * 5, 1));
        let columns = (0..5).map(|colnum| (colnum, 5));
        rows.chain(columns)
            .map(|(offset, stride)| (0..5).map(move |i| i * stride + offset))
    }

    fn is_winning(&self) -> bool {
        Self::straight_lines()
            .map(|locations| {
                locations.map(|loc| &self.tiles[loc]).collect::<Vec<_>>()
            }).any(|tiles| tiles.iter().all(|tile| tile.called))
    }

    fn call_number(&mut self, num: u32) {
        self.tiles.iter_mut().for_each(|tile| {
            if tile.number == num {
                tile.called = true;
            }
        })
    }

    fn rounds_until_completed(
        &self,
        numbers: &[u32],
    ) -> Option<(usize, u32, BingoBoard)> {
        let mut state = self.clone();
        numbers
            .iter()
            .scan(&mut state, |state, &num| {
                state.call_number(num);
                Some((state.is_winning(), num))
            })
            .enumerate()
            .filter(|(_i, (is_winning, _num))| *is_winning)
            .map(|(i, (_is_winning, num))| (i, num))
            .next()
            .map(|(i, num)| (i, num, state))
    }

    fn score(&self, last_num: u32) -> u32 {
        last_num
            * self
                .tiles
                .iter()
                .filter(|tile| !tile.called)
                .map(|tile| tile.number)
                .sum::<u32>()
    }
}

impl std::fmt::Display for BingoBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.tiles
            .iter()
            .enumerate()
            .map(|(i, tile)| {
                let end_char = if (i + 1) == self.tiles.len() {
                    ""
                } else if (i + 1) % 5 == 0 {
                    "\n"
                } else {
                    "  "
                };
                write!(f, "{}{}", tile, end_char)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }
}

impl std::fmt::Display for BingoGame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Upcoming numbers = {:?}", self.numbers_called)?;
        self.boards
            .iter()
            .enumerate()
            .map(|(i, board)| {
                let ending = if i + 1 == self.boards.len() {
                    ""
                } else {
                    "\n\n"
                };
                write!(f, "{}{}", board, ending)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }
}

impl BingoGame {
    fn find_winning_board(
        &self,
        goal: WhichBoard,
    ) -> Result<(u32, BingoBoard), Error> {
        self.boards
            .iter()
            .map(|board| {
                board
                    .rounds_until_completed(self.numbers_called.as_slice())
                    .ok_or(Error::BoardNeverWins)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .min_by_key(|(rounds, _last_num, _board)| match goal {
                WhichBoard::First => *rounds,
                WhichBoard::Last => self.numbers_called.len() - *rounds,
            })
            .map(|(_rounds, last_num, board)| (last_num, board))
            .ok_or(Error::NoWinningBoard)
    }
}

impl Puzzle for Day04 {
    const YEAR: u32 = 2021;
    const DAY: u8 = 4;
    const IMPLEMENTED: bool = true;
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = BingoGame;
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let numbers_called = lines
            .next()
            .ok_or(Error::NoneError)?
            .split(',')
            .map(|item| item.parse::<u32>())
            .collect::<Result<_, _>>()?;

        let boards = lines
            .skip(1)
            .chunks(6)
            .into_iter()
            .map(|chunk| -> Result<_, Error> {
                let tiles = chunk
                    .flat_map(|line| line.split(' '))
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse::<u32>())
                    .map_ok(|number| BingoTile {
                        number,
                        called: false,
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if tiles.len() == 25 {
                    Ok(BingoBoard { tiles })
                } else {
                    Err(Error::WrongBingoBoardSize(tiles.len()))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(BingoGame {
            numbers_called,
            boards,
        })
    }

    type Part1Result = u32;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        let (last_num, board) = parsed.find_winning_board(WhichBoard::First)?;
        Ok(board.score(last_num))
    }

    type Part2Result = u32;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        let (last_num, board) = parsed.find_winning_board(WhichBoard::Last)?;
        Ok(board.score(last_num))
    }
}
