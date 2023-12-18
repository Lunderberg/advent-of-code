use crate::prelude::Vector;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn as_vec(self) -> Vector<2, i64> {
        match self {
            Direction::Up => [0, -1].into(),
            Direction::Down => [0, 1].into(),
            Direction::Left => [-1, 0].into(),
            Direction::Right => [1, 0].into(),
        }
    }

    pub fn iter_cardinal() -> impl Iterator<Item = Direction> {
        [Self::Up, Self::Down, Self::Left, Self::Right].into_iter()
    }

    pub fn turn_right(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }

    pub fn turn_left(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
        }
    }

    pub fn reverse(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}
