#![allow(unused_imports)]
use crate::{Error, Puzzle};

use std::cmp::Reverse;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use itertools::Itertools;
use priority_queue::PriorityQueue;

pub struct Day21;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum GameState {
    Player1Victory,
    Player2Victory,
    InProgress(InProgressGameState),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct InProgressGameState {
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
    fn priority(&self) -> Reverse<u64> {
        use GameState::*;
        match self {
            Player1Victory => Reverse(u64::MAX),
            Player2Victory => Reverse(u64::MAX),
            InProgress(state) => {
                Reverse(state.players.iter().map(|p| p.score).sum())
            }
        }
    }

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
            in_progress.as_full_state()
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

    fn as_full_state(self) -> GameState {
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

impl Puzzle for Day21 {
    const YEAR: u32 = 2021;
    const DAY: u8 = 21;
    const IMPLEMENTED: bool = true;
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = InProgressGameState;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let (player1, player2) = lines
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
            winning_score: 0,
        })
    }

    type Part1Result = u64;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        let mut state = *parsed;
        state.winning_score = 1000;

        let mut die = DeterministicDie::new();

        while !state.is_finished() {
            state.take_turn(die.roll() + die.roll() + die.roll());
        }
        Ok(state.lowest_score() * die.num_times_rolled)
    }

    type Part2Result = usize;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        let mut state = *parsed;
        state.winning_score = 21;
        let state = state.as_full_state();

        let mut states: HashMap<GameState, usize> = HashMap::new();
        states.insert(state, 1);

        let mut state_queue = PriorityQueue::new();
        state_queue.push(state, state.priority());

        while state_queue
            .peek()
            .map(|(next_state, _)| !next_state.is_finished())
            .unwrap_or(false)
        {
            let state = state_queue.pop().unwrap().0;
            let counts = states.remove(&state).unwrap();

            state
                .after_dirac_turn()
                .map(|(new_state, factor)| (new_state, counts * factor))
                .for_each(|(new_state, counts)| {
                    if let Some(prev_counts) = states.get_mut(&new_state) {
                        *prev_counts += counts;
                    } else {
                        states.insert(new_state, counts);
                        state_queue.push(new_state, new_state.priority());
                    }
                });
        }

        Ok(states.into_values().max().unwrap())
    }
}
