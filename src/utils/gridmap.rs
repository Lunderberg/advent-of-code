use std::fmt::{Debug, Display, Formatter};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct GridMap<T> {
    pub x_size: usize,
    pub y_size: usize,
    values: Vec<T>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GridPos {
    index: usize,
}

#[derive(Debug)]
enum GridMapError {
    InconsistentLineSize,
    MissingValue,
    DuplicateValue,
}

pub enum Adjacency {
    Rook,
    Queen,
    Region3x3,
}

pub enum InputGridPos {
    FlatIndex(usize),
    XY(i64, i64),
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

impl From<(i64, i64)> for InputGridPos {
    fn from((x, y): (i64, i64)) -> Self {
        InputGridPos::XY(x, y)
    }
}

impl GridPos {
    pub fn as_flat(&self) -> usize {
        self.index
    }

    pub fn as_xy<T>(&self, map: &GridMap<T>) -> (i64, i64) {
        (
            (self.index % map.x_size) as i64,
            (self.index / map.x_size) as i64,
        )
    }
}

impl InputGridPos {
    fn normalize<T>(&self, map: &GridMap<T>) -> Option<GridPos> {
        use InputGridPos::*;
        match self {
            FlatIndex(index) => Some(GridPos { index: *index }),
            XY(x, y) => {
                let coordinates_valid = *x >= 0
                    && *y >= 0
                    && *x < (map.x_size as i64)
                    && *y < (map.y_size as i64);
                coordinates_valid.then_some(GridPos {
                    index: (*y as usize) * map.x_size + (*x as usize),
                })
            }
        }
    }

    fn as_xy<T>(&self, map: &GridMap<T>) -> (i64, i64) {
        use InputGridPos::*;
        match self {
            FlatIndex(index) => GridPos { index: *index }.as_xy(map),
            XY(x, y) => (*x, *y),
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
            Adjacency::Region3x3 => vec![
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 0),
                (0, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ]
            .into_iter(),
        }
    }

    pub fn adjacent(&self, i: i64, j: i64) -> impl Iterator<Item = (i64, i64)> {
        self.offsets().map(move |(di, dj)| (i + di, j + dj))
    }
}

impl<T> FromIterator<(usize, usize, T)> for GridMap<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (usize, usize, T)>,
    {
        let (xvals, yvals, values): (Vec<_>, Vec<_>, Vec<_>) =
            iter.into_iter().multiunzip();
        let x_size = xvals.iter().max().unwrap() + 1;
        let y_size = yvals.iter().max().unwrap() + 1;

        let values = xvals
            .into_iter()
            .zip(yvals.into_iter())
            .zip(values.into_iter())
            .map(|((x, y), val)| (y * x_size + x, val))
            .sorted_by_key(|(pos, _val)| *pos)
            .enumerate()
            .map(|(i, (pos, val))| {
                (i == pos).then_some(val).ok_or({
                    if i < pos {
                        GridMapError::MissingValue
                    } else {
                        GridMapError::DuplicateValue
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        Self {
            x_size,
            y_size,
            values,
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
                writeln!(f)?;
                Ok(())
            })?;
        Ok(())
    }
}

impl<T> GridMap<T> {
    fn adjacent_points_internal<P>(
        &self,
        pos: P,
        adj: Adjacency,
    ) -> impl Iterator<Item = (Option<GridPos>, (i64, i64))> + '_
    where
        P: Into<InputGridPos>,
    {
        let (x0, y0) = pos.into().as_xy(self);

        adj.offsets().map(move |(dx, dy)| {
            let y = y0 + dy;
            let x = x0 + dx;
            let gridpos = InputGridPos::XY(x, y).normalize(self);

            (gridpos, (x, y))
        })
    }

    pub fn adjacent_points<P>(
        &self,
        pos: P,
        adj: Adjacency,
    ) -> impl Iterator<Item = GridPos> + '_
    where
        P: Into<InputGridPos>,
    {
        let pos = pos.into();
        self.adjacent_points_internal(pos, adj)
            .filter_map(|(adj_pos, _xy)| adj_pos)
    }

    pub fn adjacent_values_default<P>(
        &self,
        pos: P,
        adj: Adjacency,
        default: T,
    ) -> impl Iterator<Item = T> + '_
    where
        P: Into<InputGridPos>,
        T: Clone,
    {
        let pos = pos.into();
        self.adjacent_points_internal(pos, adj)
            .map(move |(adj_pos, _xy)| {
                adj_pos
                    .map(|pos| self[pos].clone())
                    .unwrap_or_else(|| default.clone())
            })
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

    pub fn cartesian_dist2(&self, a: &GridPos, b: &GridPos) -> i64 {
        let (ax, ay) = a.as_xy(self);
        let (bx, by) = b.as_xy(self);

        (ax - bx).pow(2) + (ay - by).pow(2)
    }

    pub fn manhattan_dist(&self, a: &GridPos, b: &GridPos) -> i64 {
        let (ax, ay) = a.as_xy(self);
        let (bx, by) = b.as_xy(self);

        (ax - bx).abs() + (ay - by).abs()
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
        InputGridPos::XY((self.x_size as i64) - 1, (self.y_size as i64) - 1)
            .normalize(self)
            .unwrap()
    }
}
