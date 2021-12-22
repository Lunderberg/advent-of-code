#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

pub struct Day21;

#[derive(Debug)]
struct GameState {
    players: Vec<PlayerState>,
    turn: usize,
    die: DeterministicDie,
}

#[derive(Debug)]
struct PlayerState {
    space: u64,
    score: u64,
}

#[derive(Debug)]
struct DeterministicDie {
    prev_roll: u64,
    num_times_rolled: u64,
}

impl GameState {
    const WINNING_SCORE: u64 = 1000;
    fn take_turn(&mut self) {
        let player = &mut self.players[self.turn];
        player.advance(self.die.roll() + self.die.roll() + self.die.roll());
        self.turn = (self.turn + 1) % self.players.len();
    }

    fn is_finished(&self) -> bool {
        self.players.iter().any(|p| p.score >= Self::WINNING_SCORE)
    }

    fn lowest_score(&self) -> u64 {
        self.players.iter().map(|p| p.score).min().unwrap()
    }
}

impl DeterministicDie {
    const NUM_SIDES: u64 = 100;

    fn new() -> Self {
        Self {
            prev_roll: Self::NUM_SIDES,
            num_times_rolled: 0,
        }
    }

    fn roll(&mut self) -> u64 {
        self.prev_roll = (self.prev_roll % Self::NUM_SIDES) + 1;
        self.num_times_rolled += 1;
        self.prev_roll
    }
}

impl PlayerState {
    const TRACK_SIZE: u64 = 10;

    fn advance(&mut self, num_spaces: u64) {
        self.space = (self.space + num_spaces - 1) % Self::TRACK_SIZE + 1;
        self.score += self.space;
    }
}

impl Day21 {
    fn parse(&self) -> Result<GameState, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let players = puzzle_input
            .lines()
            .map(|line| {
                line.split(' ')
                    .last()
                    .ok_or(Error::NoneError)
                    .and_then(|s| s.parse::<u64>().map_err(|e| e.into()))
                    .map(|space| PlayerState { space, score: 0 })
            })
            .collect::<Result<_, _>>()?;

        let die = DeterministicDie::new();

        Ok(GameState {
            players,
            turn: 0,
            die,
        })
    }
}

impl Puzzle for Day21 {
    fn day(&self) -> i32 {
        21
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let mut state = self.parse()?;

        while !state.is_finished() {
            state.take_turn();
        }
        let result = state.lowest_score() * state.die.num_times_rolled;

        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result = ();
        Ok(Box::new(result))
    }
}
