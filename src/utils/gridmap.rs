use std::fmt::{Debug, Display, Formatter};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct GridMap<T> {
    x_size: usize,
    y_size: usize,
    values: Vec<T>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GridPos {
    index: usize,
}

#[derive(Debug)]
enum GridMapError {
    InconsistentLineSize,
}

pub enum Adjacency {
    Rook,
    Queen,
}

pub enum InputGridPos {
    FlatIndex(usize),
    XY(usize, usize),
}

impl From<GridPos> for InputGridPos {
    fn from(pos: GridPos) -> Self {
        InputGridPos::FlatIndex(pos.index)
    }
}

impl From<usize> for InputGridPos {
    fn from(i: usize) -> Self {
        InputGridPos::FlatIndex(i)
    }
}

impl From<(usize, usize)> for InputGridPos {
    fn from((x, y): (usize, usize)) -> Self {
        InputGridPos::XY(x, y)
    }
}

impl GridPos {
    pub fn as_flat(&self) -> usize {
        self.index
    }

    pub fn as_xy(&self, x_size: usize) -> (usize, usize) {
        (self.index % x_size, self.index / x_size)
    }
}

impl InputGridPos {
    fn normalize(&self, x_size: usize) -> GridPos {
        use InputGridPos::*;
        match self {
            FlatIndex(index) => GridPos { index: *index },
            XY(x, y) => GridPos {
                index: y * x_size + x,
            },
        }
    }
}

impl Adjacency {
    pub fn offsets(&self) -> impl Iterator<Item = (i64, i64)> {
        match self {
            Adjacency::Rook => {
                vec![(0, 1), (1, 0), (0, -1), (-1, 0)].into_iter()
            }
            Adjacency::Queen => vec![
                (0, 1),
                (1, 1),
                (1, 0),
                (1, -1),
                (0, -1),
                (-1, -1),
                (-1, 0),
                (-1, 1),
            ]
            .into_iter(),
        }
    }
}

impl<'a, T> FromIterator<&'a str> for GridMap<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let (line_length, line_num, value_results): (Vec<_>, Vec<_>, Vec<_>) =
            iter.into_iter()
                .enumerate()
                .flat_map(|(line_num, line)| {
                    let length = line.len();
                    line.chars().collect::<Vec<_>>().into_iter().map(move |c| {
                        (length, line_num, c.to_string().parse::<T>())
                    })
                })
                .multiunzip();

        let values = value_results
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let y_size = line_num.last().map_or(0, |last| last + 1);
        let x_size = line_length
            .into_iter()
            .unique()
            .exactly_one()
            .map_err(|_| GridMapError::InconsistentLineSize)
            .unwrap();

        Self {
            x_size,
            y_size,
            values,
        }
    }
}

impl<T> Display for GridMap<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.values
            .iter()
            .chunks(self.x_size)
            .into_iter()
            .try_for_each(|mut chunk| -> Result<_, std::fmt::Error> {
                chunk.try_for_each(|val| write!(f, "{}", val))?;
                write!(f, "\n")?;
                Ok(())
            })?;
        Ok(())
    }
}

impl<T> GridMap<T> {
    pub fn adjacent_points<P>(
        &self,
        pos: P,
        adj: Adjacency,
    ) -> impl Iterator<Item = GridPos>
    where
        P: Into<InputGridPos>,
    {
        let (x0, y0) = pos.into().normalize(self.x_size).as_xy(self.x_size);
        let x_size = self.x_size;
        let y_size = self.x_size;

        adj.offsets()
            .map(move |(dx, dy)| {
                let y = (y0 as i64) + dy;
                let x = (x0 as i64) + dx;
                if x >= 0
                    && y >= 0
                    && x < (x_size as i64)
                    && y < (y_size as i64)
                {
                    let pos: InputGridPos = (x as usize, y as usize).into();
                    Some(pos.normalize(x_size))
                } else {
                    None
                }
            })
            .flatten()
    }

    pub fn iter(&self) -> impl Iterator<Item = (GridPos, &T)> {
        self.values
            .iter()
            .enumerate()
            .map(move |(index, val)| (GridPos { index }, val))
    }

    pub fn into_iter(self) -> impl Iterator<Item = (GridPos, T)> {
        self.values
            .into_iter()
            .enumerate()
            .map(move |(index, val)| (GridPos { index }, val))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (GridPos, &mut T)> {
        self.values
            .iter_mut()
            .enumerate()
            .map(move |(index, val)| (GridPos { index }, val))
    }

    pub fn cartesian_dist2(&self, a: &GridPos, b: &GridPos) -> usize {
        let (ax, ay) = a.as_xy(self.x_size);
        let (bx, by) = b.as_xy(self.x_size);

        // Have .abs_diff()
        // https://github.com/rust-lang/rust/issues/89492 would be
        // really nice here.
        // ax.abs_diff(bx).pow(2) + ay.abs_diff(by).pow(2)

        let x_min = ax.min(bx);
        let x_max = ax.max(bx);
        let y_min = ay.min(by);
        let y_max = ay.max(by);

        (x_max - x_min).pow(2) + (y_max - y_min).pow(2)
    }

    pub fn manhattan_dist(&self, a: &GridPos, b: &GridPos) -> usize {
        let (ax, ay) = a.as_xy(self.x_size);
        let (bx, by) = b.as_xy(self.x_size);

        // Have .abs_diff()
        // https://github.com/rust-lang/rust/issues/89492 would be
        // really nice here.
        // ax.abs_diff(bx) + ay.abs_diff(by)

        let x_min = ax.min(bx);
        let x_max = ax.max(bx);
        let y_min = ay.min(by);
        let y_max = ay.max(by);

        (x_max - x_min) + (y_max - y_min)
    }
}

impl<T> Index<GridPos> for GridMap<T> {
    type Output = T;
    fn index(&self, pos: GridPos) -> &T {
        &self.values[pos.index]
    }
}

impl<T> IndexMut<GridPos> for GridMap<T> {
    fn index_mut(&mut self, pos: GridPos) -> &mut T {
        &mut self.values[pos.index]
    }
}

impl<T> GridMap<T> {
    pub fn top_left(&self) -> GridPos {
        GridPos { index: 0 }
    }

    pub fn bottom_right(&self) -> GridPos {
        InputGridPos::XY(self.x_size - 1, self.y_size - 1)
            .normalize(self.x_size)
    }
}
