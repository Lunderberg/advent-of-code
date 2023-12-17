use std::{collections::HashSet, fmt::Display};

use aoc_utils::prelude::*;

pub struct HeatLossMap(GridMap<u8>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl Direction {
    fn as_vec(self) -> Vector<2, i64> {
        match self {
            Direction::Up => [0, -1].into(),
            Direction::Down => [0, 1].into(),
            Direction::Left => [-1, 0].into(),
            Direction::Right => [1, 0].into(),
        }
    }

    fn turn_right(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }
    fn turn_left(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
        }
    }
}

impl Display for HeatLossMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CrucibleState {
    pos: Vector<2, i64>,
    dir: Direction,
    current_straight_line: u8,
    is_ultra: bool,
}

impl EdgeWeightedGraph<CrucibleState> for HeatLossMap {
    fn connections_from<'a>(
        &'a self,
        node: &'a CrucibleState,
    ) -> impl Iterator<Item = (CrucibleState, u64)> + '_ {
        let pos = node.pos;
        let dir = node.dir;
        let current_straight_line = node.current_straight_line;
        let is_ultra = node.is_ultra;

        [dir, dir.turn_left(), dir.turn_right()]
            .into_iter()
            .filter(move |&new_dir| {
                (is_ultra
                    && ((new_dir != dir && 4 <= current_straight_line)
                        || (new_dir == dir && current_straight_line < 10)))
                    || (!is_ultra
                        && ((new_dir != dir)
                            || (new_dir == dir && current_straight_line < 3)))
            })
            .map(move |new_dir| CrucibleState {
                pos: pos + new_dir.as_vec(),
                dir: new_dir,
                current_straight_line: if dir == new_dir {
                    current_straight_line + 1
                } else {
                    1
                },
                is_ultra,
            })
            .filter_map(|state| {
                self.0.get(state.pos).map(|value| (state, *value as u64))
            })
    }
}

impl HeatLossMap {
    fn min_heat_loss(&self, is_ultra: bool) -> u64 {
        let (height, width) = self.0.shape();
        let goal: Vector<2, i64> = [height as i64 - 1, width as i64 - 1].into();

        let manhattan_heuristic = |state: &CrucibleState| -> Option<u64> {
            Some(
                (state.pos - goal)
                    .map(|delta| delta.unsigned_abs())
                    .into_iter()
                    .sum::<u64>(),
            )
        };

        let initial =
            [Direction::Right, Direction::Down].map(|dir| CrucibleState {
                pos: [0, 0].into(),
                dir,
                current_straight_line: 0,
                is_ultra,
            });

        let states: Vec<_> = self
            .iter_a_star(initial, manhattan_heuristic)
            //.take_while_inclusive(|search_item| search_item.item.pos != goal)
            .take_while_inclusive(|search_item| {
                search_item.item.pos != goal
                    || (is_ultra && search_item.item.current_straight_line < 4)
            })
            .collect();

        if states.last().unwrap().item.pos != goal {
            panic!("No path to goal location");
        }

        let path: HashSet<Vector<2, i64>> =
            std::iter::successors(states.last(), |search_item| {
                search_item.backref.map(|i| &states[i])
            })
            .map(|search_item| search_item.item.pos)
            .collect();

        println!(
            "{}",
            self.0.map(|(pos, val): (Vector<2, i64>, &u8)| {
                let style = if path.contains(&pos) {
                    console::Style::new().green().bright().bold()
                } else {
                    console::Style::new().red()
                };
                style.apply_to(val)
            })
        );

        let heat_loss = states.last().unwrap().total_dist;

        // let heat_loss = self
        //     .iter_a_star([initial], manhattan_heuristic)
        //     .find(|search_item| search_item.item.pos == goal)
        //     .map(|search_item| search_item.total_dist)
        //     .expect("No path to goal location");

        heat_loss
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = HeatLossMap;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let map = lines
            .enumerate()
            .flat_map(|(y, line)| {
                line.chars()
                    .enumerate()
                    .map(move |(x, c)| (x, y, c.to_digit(10).unwrap() as u8))
            })
            .collect();
        Ok(HeatLossMap(map))
    }

    fn part_1(
        heat_loss_map: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(heat_loss_map.min_heat_loss(false))
    }

    fn part_2(
        heat_loss_map: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(heat_loss_map.min_heat_loss(true))
    }
}
