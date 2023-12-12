use crate::extensions::CharIterLocExt;
use crate::geometry::Vector;

use std::fmt::{Debug, Display, Formatter};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

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
pub enum GridMapError {
    InconsistentLineSize,
    MissingValue,
    DuplicateValue,
    InvalidLinearIndex,
    InvalidXYIndex,
}

pub enum Adjacency {
    Rook,
    Queen,
    Region3x3,
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

pub trait IntoGridPos {
    fn into_grid_pos<T>(
        self,
        map: &GridMap<T>,
    ) -> Result<GridPos, GridMapError>;
}

pub trait FromGridPos<'map, T> {
    fn from_grid_pos(pos: GridPos, map: &'map GridMap<T>) -> Self;
}

impl IntoGridPos for GridPos {
    fn into_grid_pos<T>(self, _: &GridMap<T>) -> Result<GridPos, GridMapError> {
        Ok(self)
    }
}

impl<'map, T> FromGridPos<'map, T> for GridPos {
    fn from_grid_pos(pos: GridPos, _: &'map GridMap<T>) -> Self {
        pos
    }
}

impl IntoGridPos for usize {
    fn into_grid_pos<T>(
        self,
        map: &GridMap<T>,
    ) -> Result<GridPos, GridMapError> {
        let (width, height) = map.shape();
        if self < width * height {
            Ok(GridPos { index: self })
        } else {
            Err(GridMapError::InvalidLinearIndex)
        }
    }
}

impl<'map, T> FromGridPos<'map, T> for usize {
    fn from_grid_pos(pos: GridPos, _: &'map GridMap<T>) -> Self {
        pos.index
    }
}

impl IntoGridPos for (i64, i64) {
    fn into_grid_pos<T>(
        self,
        map: &GridMap<T>,
    ) -> Result<GridPos, GridMapError> {
        let (width, height) = map.shape();
        let (x, y) = self;
        let coordinates_valid =
            x >= 0 && y >= 0 && x < (width as i64) && y < (height as i64);

        if coordinates_valid {
            let x = x as usize;
            let y = y as usize;
            Ok(GridPos {
                index: y * width + x,
            })
        } else {
            Err(GridMapError::InvalidXYIndex)
        }
    }
}

impl<'map, T> FromGridPos<'map, T> for (i64, i64) {
    fn from_grid_pos(pos: GridPos, map: &'map GridMap<T>) -> Self {
        let (width, _) = map.shape();
        let x = pos.index.rem_euclid(width) as i64;
        let y = pos.index.div_euclid(width) as i64;
        (x, y)
    }
}

impl IntoGridPos for Vector<2, i64> {
    fn into_grid_pos<T>(
        self,
        map: &GridMap<T>,
    ) -> Result<GridPos, GridMapError> {
        (self.x(), self.y()).into_grid_pos(map)
    }
}

impl<'map, T> FromGridPos<'map, T> for Vector<2, i64> {
    fn from_grid_pos(pos: GridPos, map: &'map GridMap<T>) -> Self {
        let (x, y): (i64, i64) = FromGridPos::from_grid_pos(pos, map);
        (x, y).into()
    }
}

impl<'map, 'item, Pos, T> FromGridPos<'map, T> for (Pos, &'item T)
where
    'map: 'item,
    Pos: FromGridPos<'map, T>,
{
    fn from_grid_pos(pos: GridPos, map: &'map GridMap<T>) -> Self {
        (FromGridPos::from_grid_pos(pos, map), &map[pos])
    }
}

impl<'map, 'item, T> FromGridPos<'map, T> for &'item T
where
    'map: 'item,
{
    fn from_grid_pos(pos: GridPos, map: &'map GridMap<T>) -> Self {
        &map[pos]
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

impl<T> FromIterator<(Vector<2, i64>, T)> for GridMap<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (Vector<2, i64>, T)>,
    {
        iter.into_iter()
            .map(|(pos, tile)| {
                let (x, y) = pos.into();
                (x as usize, y as usize, tile)
            })
            .collect()
    }
}

impl<'a, T> FromIterator<&'a str> for GridMap<T>
where
    char: TryInto<T>,
    <char as TryInto<T>>::Error: Debug,
{
    fn from_iter<I>(lines: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        lines
            .into_iter()
            .flat_map(|line| line.chars().chain(std::iter::once('\n')))
            .collect()
    }
}

impl<T> FromIterator<char> for GridMap<T>
where
    char: TryInto<T>,
    <char as TryInto<T>>::Error: Debug,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = char>,
    {
        iter.into_iter()
            .with_char_loc()
            .filter(|(_, c)| *c != '\n')
            .map(|(loc, c)| {
                c.try_into().map(move |t| (loc.col_num, loc.line_num, t))
            })
            .collect::<Result<Self, _>>()
            .unwrap()
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
                chunk.try_for_each(|val| write!(f, "{val}"))?;
                writeln!(f)?;
                Ok(())
            })?;
        Ok(())
    }
}

impl<T> GridMap<T> {
    pub fn new_uniform(x_size: usize, y_size: usize, value: T) -> Self
    where
        T: Clone,
    {
        let mut values = Vec::new();
        values.resize(x_size * y_size, value);
        Self {
            x_size,
            y_size,
            values,
        }
    }

    pub fn is_valid(&self, index: impl IntoGridPos) -> bool {
        index.into_grid_pos(self).is_ok()
    }

    pub fn get(&self, index: impl IntoGridPos) -> Option<&T> {
        index
            .into_grid_pos(self)
            .ok()
            .map(|grid_pos| &self.values[grid_pos.index])
    }

    pub fn get_mut(&mut self, index: impl IntoGridPos) -> Option<&mut T> {
        index
            .into_grid_pos(self)
            .ok()
            .map(move |grid_pos| &mut self.values[grid_pos.index])
    }

    pub fn shape(&self) -> (usize, usize) {
        (self.x_size, self.y_size)
    }

    pub fn grid_pos(&self, arg: impl IntoGridPos) -> Option<GridPos> {
        arg.into_grid_pos(self).ok()
    }

    fn adjacent_points_internal(
        &self,
        pos: impl IntoGridPos,
        adj: Adjacency,
    ) -> Result<impl Iterator<Item = Option<GridPos>> + '_, GridMapError> {
        let (x0, y0) = pos.into_grid_pos(self)?.as_xy(self);

        let iter = adj.offsets().map(move |(dx, dy)| {
            let y = y0 + dy;
            let x = x0 + dx;
            (x, y).into_grid_pos(self).ok()
        });
        Ok(iter)
    }

    pub fn adjacent_points(
        &self,
        pos: impl IntoGridPos,
        adj: Adjacency,
    ) -> impl Iterator<Item = GridPos> + '_ {
        self.adjacent_points_internal(pos, adj)
            .unwrap()
            .flatten()
            .map(|pos| FromGridPos::from_grid_pos(pos, self))
    }

    pub fn adjacent_values_default(
        &self,
        pos: impl IntoGridPos,
        adj: Adjacency,
        default: T,
    ) -> impl Iterator<Item = T> + '_
    where
        T: Clone,
    {
        self.adjacent_points_internal(pos, adj)
            .unwrap()
            .map(move |opt_pos| {
                opt_pos.map_or_else(|| default.clone(), |pos| self[pos].clone())
            })
    }

    pub fn iter<'map, Item>(&'map self) -> impl Iterator<Item = Item> + '_
    where
        Item: FromGridPos<'map, T>,
    {
        (0..self.values.len())
            .map(|index| GridPos { index })
            .map(|pos| FromGridPos::from_grid_pos(pos, self))
    }

    pub fn iter_item(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }

    pub fn iter_pos(&self) -> impl Iterator<Item = (GridPos, &T)> {
        self.iter()
    }

    pub fn iter_vec(&self) -> impl Iterator<Item = (Vector<2, i64>, &T)> {
        self.iter()
    }

    pub fn iter_pos_mut(&mut self) -> impl Iterator<Item = (GridPos, &mut T)> {
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
        let (width, height) = self.shape();
        let x = (width - 1) as i64;
        let y = (height - 1) as i64;
        (x, y).into_grid_pos(self).unwrap()
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

            (x, y)
                .into_grid_pos(self)
                .map(|gridpos| (gridpos, &self[gridpos]))
                .ok()
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
            (x, y)
                .into_grid_pos(self)
                .map(|gridpos| (gridpos, &self[gridpos]))
                .ok()
        })
    }

    pub fn iter_rect(
        &self,
        corner_a: GridPos,
        corner_b: GridPos,
    ) -> impl Iterator<Item = (GridPos, &T)> + '_ {
        let (x_a, y_a) = corner_a.as_xy(self);
        let (x_b, y_b) = corner_b.as_xy(self);
        (y_a..=y_b)
            .flat_map(move |y| (x_a..=x_b).map(move |x| (x, y)))
            .filter_map(|xy| xy.into_grid_pos(self).ok())
            .map(|gridpos| (gridpos, &self[gridpos]))
    }

    pub fn map<'map, Arg, F, U>(&'map self, mut func: F) -> GridMap<U>
    where
        Arg: FromGridPos<'map, T>,
        F: FnMut(Arg) -> U,
    {
        self.iter()
            .map(|grid_pos: GridPos| {
                let pos: Vector<2> = FromGridPos::from_grid_pos(grid_pos, self);
                let arg: Arg = FromGridPos::from_grid_pos(grid_pos, self);
                (pos, func(arg))
            })
            .collect()
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
            .scan(0, |expected_pos, (iter_order, (pos, val))| {
                let num_before = pos - *expected_pos;
                *expected_pos = pos + 1;
                let num_after = match iter_order {
                    itertools::Position::Last => {
                        let total = (x_size * y_size) as usize;
                        total - (pos + 1)
                    }
                    _ => 0,
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

impl<T, Arg: IntoGridPos> Index<Arg> for GridMap<T> {
    type Output = T;
    fn index(&self, pos: Arg) -> &T {
        let grid_pos: GridPos = pos.into_grid_pos(self).unwrap();
        &self.values[grid_pos.index]
    }
}

impl<T, Arg: IntoGridPos> IndexMut<Arg> for GridMap<T> {
    fn index_mut(&mut self, pos: Arg) -> &mut T {
        let grid_pos: GridPos = pos.into_grid_pos(self).unwrap();
        &mut self.values[grid_pos.index]
    }
}
