#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use itertools::Itertools;

pub struct Day21;

#[derive(Debug)]
struct InProgressGameState {
    winning_score: u64,
    players: [PlayerState; 2],
    turn: usize,
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

impl InProgressGameState {
    fn take_turn(&mut self, advance_by: u64) {
        self.players[self.turn].advance(advance_by);
        self.turn = (self.turn + 1) % self.players.len();
    }

    fn is_finished(&self) -> bool {
        self.players.iter().any(|p| p.score >= self.winning_score)
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

    fn advance(&mut self, advance_by: u64) {
        self.space = (self.space + advance_by - 1) % Self::TRACK_SIZE + 1;
        self.score += self.space;
    }
}

impl Day21 {
    fn parse_starting_state(
        &self,
        winning_score: u64,
    ) -> Result<InProgressGameState, Error> {
        let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        //let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let (player1, player2) = puzzle_input
            .lines()
            .map(|line| {
                line.split(' ')
                    .last()
                    .ok_or(Error::NoneError)
                    .and_then(|s| s.parse::<u64>().map_err(|e| e.into()))
                    .map(|space| PlayerState { space, score: 0 })
            })
            .tuples()
            .exactly_one()?;

        let players = [player1?, player2?];

        Ok(InProgressGameState {
            players,
            turn: 0,
            winning_score,
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
        let winning_score = 1000;
        let mut state = self.parse_starting_state(winning_score)?;

        let mut die = DeterministicDie::new();

        while !state.is_finished() {
            state.take_turn(die.roll() + die.roll() + die.roll());
        }
        let result = state.lowest_score() * die.num_times_rolled;

        Ok(Box::new(result))
    }

    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result = ();
        Ok(Box::new(result))
    }
}
