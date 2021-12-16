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

#[derive(Debug, Clone, Copy)]
pub enum GridPos {
    FlatIndex(usize),
    XY(usize, usize),
}

#[derive(Debug)]
enum GridMapError {
    InconsistentLineSize,
}

pub enum Adjacency {
    Rook,
    Queen,
}

impl From<usize> for GridPos {
    fn from(i: usize) -> Self {
        GridPos::FlatIndex(i)
    }
}

impl From<(usize, usize)> for GridPos {
    fn from((x, y): (usize, usize)) -> Self {
        GridPos::XY(x, y)
    }
}

impl GridPos {
    pub fn normalize<T>(&self, map: &GridMap<T>) -> usize {
        self.as_flat(map.x_size)
    }
    pub fn as_flat(&self, x_size: usize) -> usize {
        match self {
            GridPos::FlatIndex(i) => *i,
            GridPos::XY(x, y) => y * x_size + x,
        }
    }

    pub fn as_xy(&self, x_size: usize) -> (usize, usize) {
        match self {
            GridPos::FlatIndex(i) => (i % x_size, i / x_size),
            GridPos::XY(x, y) => (*x, *y),
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
        P: Into<GridPos>,
    {
        let (x0, y0) = pos.into().as_xy(self.x_size);
        let x_size = self.x_size as i64;
        let y_size = self.x_size as i64;

        adj.offsets()
            .map(move |(dx, dy)| {
                let y = (y0 as i64) + dy;
                let x = (x0 as i64) + dx;
                if x >= 0 && y >= 0 && x < x_size && y < y_size {
                    Some(GridPos::XY(x as usize, y as usize))
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
            .map(move |(i, val)| (GridPos::FlatIndex(i), val))
    }

    pub fn into_iter(self) -> impl Iterator<Item = (GridPos, T)> {
        self.values
            .into_iter()
            .enumerate()
            .map(move |(i, val)| (GridPos::FlatIndex(i), val))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (GridPos, &mut T)> {
        self.values
            .iter_mut()
            .enumerate()
            .map(move |(i, val)| (GridPos::FlatIndex(i), val))
    }
}

impl<T> Index<GridPos> for GridMap<T> {
    type Output = T;
    fn index(&self, pos: GridPos) -> &T {
        &self.values[pos.as_flat(self.x_size)]
    }
}

impl<T> IndexMut<GridPos> for GridMap<T> {
    fn index_mut(&mut self, pos: GridPos) -> &mut T {
        &mut self.values[pos.as_flat(self.x_size)]
    }
}
