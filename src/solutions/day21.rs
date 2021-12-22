#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

pub struct Day21;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum GameState {
    Player1Victory,
    Player2Victory,
    InProgress(InProgressGameState),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct InProgressGameState {
    winning_score: u64,
    players: [PlayerState; 2],
    turn: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct PlayerState {
    space: u64,
    score: u64,
}

#[derive(Debug)]
struct DeterministicDie {
    prev_roll: u64,
    num_times_rolled: u64,
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use GameState::*;
        match self {
            InProgress(in_progress) => write!(f, "{}", in_progress),
            Player1Victory => write!(f, "Player1Victory"),
            Player2Victory => write!(f, "Player2Victory"),
        }
    }
}

impl Display for InProgressGameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let (p1_active, p2_active) =
            if self.turn == 0 { ("*", "") } else { ("", "*") };
        write!(
            f,
            "{}{}, {}{}",
            p1_active, self.players[0], p2_active, self.players[1]
        )
    }
}

impl Display for PlayerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} @ {}", self.score, self.space)
    }
}

impl GameState {
    fn after_dirac_turn(&self) -> impl Iterator<Item = (Self, usize)> + '_ {
        (0..3)
            .map(|_roll_num| (1..=3))
            .multi_cartesian_product()
            .map(|combo| combo.into_iter().sum::<u64>())
            .counts()
            .into_iter()
            .map(move |(advance_by, counts)| {
                (self.after_turn(advance_by), counts)
            })
    }

    fn after_turn(&self, advance_by: u64) -> Self {
        if let GameState::InProgress(mut in_progress) = self {
            in_progress.take_turn(advance_by);
            in_progress.as_victory()
        } else {
            *self
        }
    }

    fn is_finished(&self) -> bool {
        use GameState::*;
        match self {
            Player1Victory => true,
            Player2Victory => true,
            InProgress(_) => false,
        }
    }
}

impl InProgressGameState {
    fn take_turn(&mut self, advance_by: u64) {
        self.players[self.turn].advance(advance_by);
        self.turn = (self.turn + 1) % self.players.len();
    }

    fn as_victory(self) -> GameState {
        if self.players[0].score >= self.winning_score {
            GameState::Player1Victory
        } else if self.players[1].score >= self.winning_score {
            GameState::Player2Victory
        } else {
            GameState::InProgress(self)
        }
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
        let winning_score = 21;
        let state = self.parse_starting_state(winning_score)?;

        let mut states: HashMap<GameState, usize> = HashMap::new();
        states.insert(GameState::InProgress(state), 1);

        let mut iteration = 0;

        let print_it = |header: &str, states: &HashMap<GameState, usize>| {
            println!("--------------{}--------------", header);
            states
                .iter()
                .sorted_by_key(|(_state, counts)| *counts)
                .for_each(|(state, counts)| {
                    println!("{} counts for {}", counts, state)
                });
        };

        while states.iter().any(|(state, _count)| !state.is_finished()) {
            print_it(&format!("After {} Iterations", iteration), &states);

            iteration += 1;

            states = states
                .iter()
                .flat_map(|(state, counts)| {
                    state.after_dirac_turn().map(move |(new_state, factor)| {
                        (new_state, counts * factor)
                    })
                })
                .into_grouping_map()
                .sum();
            if iteration >= 2 {
                break;
            }
        }

        print_it("Final", &states);

        //let result = states.into_values().max().unwrap();
        let result = ();
        Ok(Box::new(result))
    }
}
