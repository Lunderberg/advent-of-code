use crate::utils::Error;

use std::ops::{Add, Index, Mul, Sub};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Vector3([i64; 3]);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Mat3([i64; 9]);

impl FromStr for Vector3 {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let mut parsed_values = s.split(',').map(|s| s.parse::<i64>());

        let mut values = [0; 3];
        let mut value_iter = values.iter_mut();

        value_iter
            .by_ref()
            .zip(parsed_values.by_ref())
            .try_for_each(|(out, val)| -> Result<_, Error> {
                *out = val?;
                Ok(())
            })?;

        if parsed_values.next().is_some() {
            Err(Error::TooManyValues)
        } else if value_iter.next().is_some() {
            Err(Error::NotEnoughValues)
        } else {
            Ok(Self(values))
        }
    }
}

impl Add for Vector3 {
    type Output = Vector3;
    fn add(self, rhs: Vector3) -> Self {
        let mut values = [0; 3];
        self.0
            .iter()
            .zip(rhs.0.iter())
            .map(|(a, b)| a + b)
            .zip(values.iter_mut())
            .for_each(|(val, out)| {
                *out = val;
            });

        Self(values)
    }
}

impl Sub for Vector3 {
    type Output = Vector3;
    fn sub(self, rhs: Vector3) -> Self {
        let mut values = [0; 3];
        self.0
            .iter()
            .zip(rhs.0.iter())
            .map(|(a, b)| a - b)
            .zip(values.iter_mut())
            .for_each(|(val, out)| {
                *out = val;
            });

        Self(values)
    }
}

impl Index<usize> for Vector3 {
    type Output = i64;
    fn index(&self, index: usize) -> &i64 {
        &self.0[index]
    }
}

impl Index<(usize, usize)> for Mat3 {
    type Output = i64;
    fn index(&self, index: (usize, usize)) -> &i64 {
        match index {
            (0..=2, 0..=2) => &self.0[3 * index.1 + index.0],
            _ => panic!("Mat3 indices must be < 3"),
        }
    }
}

impl Mul for Mat3 {
    type Output = Mat3;
    fn mul(self, rhs: Mat3) -> Self {
        let mut values = [0; 9];
        values
            .iter_mut()
            .enumerate()
            .map(|(index, out)| (index % 3, index / 3, out))
            .map(|(i, j, out)| {
                ((0..3).map(|k| self[(i, k)] * rhs[(k, j)]).sum(), out)
            })
            .for_each(|(val, out)| {
                *out = val;
            });
        Self(values)
    }
}

impl Mul<Vector3> for Mat3 {
    type Output = Vector3;
    fn mul(self, rhs: Vector3) -> Vector3 {
        let mut values = [0; 3];
        values
            .iter_mut()
            .enumerate()
            .map(|(i, out)| ((0..3).map(|k| self[(i, k)] * rhs[k]).sum(), out))
            .for_each(|(val, out)| {
                *out = val;
            });
        Vector3(values)
    }
}

impl Vector3 {
    pub fn new(arr: [i64; 3]) -> Self {
        Self(arr)
    }

    pub fn iter(&self) -> impl Iterator<Item = i64> + '_ {
        self.0.iter().copied()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut i64> + '_ {
        self.0.iter_mut()
    }

    pub fn x(&self) -> i64 {
        self.0[0]
    }

    pub fn y(&self) -> i64 {
        self.0[1]
    }

    pub fn z(&self) -> i64 {
        self.0[2]
    }

    pub fn dist2(&self, other: &Self) -> i64 {
        self.iter()
            .zip(other.iter())
            .map(|(a, b)| (a - b).pow(2))
            .sum()
    }

    pub fn manhattan_dist(&self, other: &Self) -> i64 {
        self.iter()
            .zip(other.iter())
            .map(|(a, b)| (a - b).abs())
            .sum()
    }
}

impl Mat3 {
    pub fn identity() -> Self {
        Self([1, 0, 0, 0, 1, 0, 0, 0, 1])
    }

    pub fn rotate_x() -> Self {
        Self([1, 0, 0, 0, 0, 1, 0, -1, 0])
    }

    pub fn rotate_y() -> Self {
        Self([0, 0, 1, 0, 1, 0, -1, 0, 0])
    }

    pub fn rotate_z() -> Self {
        Self([0, 1, 0, -1, 0, 0, 0, 0, 1])
    }

    pub fn pow(&self, power: usize) -> Self {
        (0..power).fold(Self::identity(), |cum_prod, _i| *self * cum_prod)
    }

    pub fn iter_90degrees() -> impl Iterator<Item = Self> {
        (0..=2)
            .flat_map(|alpha| {
                let max_beta = match alpha {
                    0 => 0,
                    1 => 3,
                    2 => 0,
                    _ => panic!("Math is broken"),
                };
                (0..=max_beta).map(move |beta| (alpha, beta))
            })
            .flat_map(|(alpha, beta)| {
                (0..=3).map(move |gamma| (alpha, beta, gamma))
            })
            .map(|(alpha, beta, gamma)| {
                Mat3::rotate_z().pow(beta)
                    * Mat3::rotate_x().pow(alpha)
                    * Mat3::rotate_z().pow(gamma)
            })
    }
}
