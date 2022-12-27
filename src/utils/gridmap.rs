use crate::utils::geometry::Vector;

use std::fmt::{Debug, Display, Formatter};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl From<Vector<2, i64>> for InputGridPos {
    fn from(value: Vector<2, i64>) -> Self {
        let (x, y) = value.into();
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

    pub fn as_vec<T>(&self, map: &GridMap<T>) -> Vector<2, i64> {
        self.as_xy(map).into()
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
                coordinates_valid.then(|| GridPos {
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
        let tuples: Vec<(usize, usize, T)> = iter.into_iter().collect();
        let x_size = tuples.iter().map(|(x, _, _)| x).max().unwrap() + 1;
        let y_size = tuples.iter().map(|(_, y, _)| y).max().unwrap() + 1;

        let values = tuples
            .into_iter()
            .map(|(x, y, val)| (y * x_size + x, val))
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
    pub fn is_valid<Arg: Into<InputGridPos>>(&self, index: Arg) -> bool {
        index.into().normalize(&self).is_some()
    }

    pub fn get<Arg: Into<InputGridPos>>(&self, index: Arg) -> Option<&T> {
        index
            .into()
            .normalize(&self)
            .map(|grid_pos| &self.values[grid_pos.index])
    }

    pub fn get_mut<Arg: Into<InputGridPos>>(
        &mut self,
        index: Arg,
    ) -> Option<&mut T> {
        index
            .into()
            .normalize(&self)
            .map(move |grid_pos| &mut self.values[grid_pos.index])
    }

    pub fn shape(&self) -> (usize, usize) {
        (self.x_size, self.y_size)
    }

    pub fn grid_pos<Arg: Into<InputGridPos>>(
        &self,
        arg: Arg,
    ) -> Option<GridPos> {
        arg.into().normalize(&self)
    }

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

    // TODO: Maybe this would be more convenient to have as the default?
    pub fn iter_vec(&self) -> impl Iterator<Item = (Vector<2, i64>, &T)> {
        self.iter().map(move |(pos, val)| (pos.as_vec(&self), val))
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

    pub fn top_left(&self) -> GridPos {
        GridPos { index: 0 }
    }

    pub fn bottom_right(&self) -> GridPos {
        InputGridPos::XY((self.x_size as i64) - 1, (self.y_size as i64) - 1)
            .normalize(self)
            .unwrap()
    }

    pub fn iter_ray(
        &self,
        start: GridPos,
        step: impl Into<(i64, i64)>,
    ) -> impl Iterator<Item = (GridPos, &T)> + '_ {
        let step = step.into();
        std::iter::successors(Some((start, &self[start])), move |(prev, _)| {
            let (prev_x, prev_y) = prev.as_xy(self);
            let x = prev_x + step.0;
            let y = prev_y + step.1;
            InputGridPos::XY(x, y)
                .normalize(self)
                .map(|gridpos| (gridpos, &self[gridpos]))
        })
    }

    pub fn iter_ray_wrapping(
        &self,
        start: GridPos,
        step: impl Into<(i64, i64)>,
    ) -> impl Iterator<Item = (GridPos, &T)> + '_ {
        let step = step.into();
        std::iter::successors(Some((start, &self[start])), move |(prev, _)| {
            let (prev_x, prev_y) = prev.as_xy(self);
            let x = (prev_x + step.0).rem_euclid(self.x_size as i64);
            let y = (prev_y + step.1).rem_euclid(self.y_size as i64);
            InputGridPos::XY(x, y)
                .normalize(self)
                .map(|gridpos| (gridpos, &self[gridpos]))
        })
    }
}

pub trait CollectResizedGridMap<T> {
    fn collect_resized_grid_map(self, default: T) -> GridMap<T>;
}

impl<T, Iter: Iterator<Item = (Vector<2, i64>, T)>> CollectResizedGridMap<T>
    for Iter
where
    T: Clone,
{
    fn collect_resized_grid_map(self, default: T) -> GridMap<T> {
        let tuples: Vec<(Vector<2, i64>, T)> = self.collect();
        let (xmin, xmax) = tuples
            .iter()
            .map(|(p, _)| p.x())
            .minmax()
            .into_option()
            .unwrap();
        let (ymin, ymax) = tuples
            .iter()
            .map(|(p, _)| p.y())
            .minmax()
            .into_option()
            .unwrap();
        let x_size = (xmax - xmin) + 1;
        let y_size = (ymax - ymin) + 1;

        let values = tuples
            .into_iter()
            .map(|(pos, val)| {
                let index = (pos.y() - ymin) * x_size + (pos.x() - xmin);
                let index = index as usize;
                (index, val)
            })
            .sorted_by_key(|(pos, _val)| *pos)
            .with_position()
            .scan(0, |expected_pos, iter_order| {
                let is_last =
                    matches!(iter_order, itertools::Position::Last(_));
                let (pos, val) = iter_order.into_inner();

                let num_before = pos - *expected_pos;
                *expected_pos = pos + 1;
                let num_after = if is_last {
                    let total = (x_size * y_size) as usize;
                    total - (pos + 1)
                } else {
                    0
                };

                Some(
                    std::iter::empty()
                        .chain(
                            std::iter::repeat(default.clone()).take(num_before),
                        )
                        .chain(std::iter::once(val))
                        .chain(
                            std::iter::repeat(default.clone()).take(num_after),
                        ),
                )
            })
            .flatten()
            .collect();

        let x_size = x_size as usize;
        let y_size = y_size as usize;

        GridMap {
            x_size,
            y_size,
            values,
        }
    }
}

pub struct GridMapIterator<T> {
    inner: std::iter::Enumerate<std::vec::IntoIter<T>>,
}

impl<T> Iterator for GridMapIterator<T> {
    type Item = (GridPos, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(index, val)| (GridPos { index }, val))
    }
}

impl<T> IntoIterator for GridMap<T> {
    type Item = (GridPos, T);

    type IntoIter = GridMapIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = self.values.into_iter().enumerate();
        Self::IntoIter { inner }
    }
}

impl<T, Arg: Into<InputGridPos>> Index<Arg> for GridMap<T> {
    type Output = T;
    fn index(&self, pos: Arg) -> &T {
        let input: InputGridPos = pos.into();
        let grid_pos: GridPos = input.normalize(&self).unwrap();
        &self.values[grid_pos.index]
    }
}

impl<T, Arg: Into<InputGridPos>> IndexMut<Arg> for GridMap<T> {
    fn index_mut(&mut self, pos: Arg) -> &mut T {
        let input: InputGridPos = pos.into();
        let grid_pos: GridPos = input.normalize(&self).unwrap();
        &mut self.values[grid_pos.index]
    }
}
