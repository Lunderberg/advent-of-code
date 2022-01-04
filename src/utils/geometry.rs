use crate::Error;

use std::ops;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Vector<const N: usize>([i64; N]);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Matrix<const N: usize, const M: usize>([[i64; M]; N]);

macro_rules! elementwise_unary_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<const N: usize, const M: usize> ops::$trait for Matrix<N, M> {
            type Output = Self;
            fn $method(self) -> Self {
                Self(self.0.map(|row| row.map(|val| $op val)))
            }
        }

        impl<const N: usize> ops::$trait for Vector<N> {
            type Output = Self;
            fn $method(self) -> Self {
                Self(self.0.map(|val| $op val))
            }
        }
    };
}

macro_rules! elementwise_scalar_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<const N: usize, const M: usize> ops::$trait<i64> for Matrix<N, M> {
            type Output = Self;
            fn $method(self, rhs: i64) -> Self {
                Self(self.0.map(|row| row.map(|val| val $op rhs)))
            }
        }

        impl<const N: usize> ops::$trait<i64> for Vector<N> {
            type Output = Self;
            fn $method(self, rhs: i64) -> Self {
                Self(self.0.map(|val| val $op rhs))
            }
        }
    };
}

macro_rules! elementwise_binary_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<const N: usize, const M: usize> ops::$trait for Matrix<N, M> {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self {
                let mut result = Self::zero();
                self.iter_flat()
                    .zip(rhs.iter_flat())
                    .map(|(a, b)| a $op b)
                    .zip(result.iter_flat_mut())
                    .for_each(|(val, out)| {
                        *out = val;
                    });

                result
            }
        }

        impl<const N: usize> ops::$trait for Vector<N> {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self {
                let mut result = Self::zero();
                self.iter()
                    .zip(rhs.iter())
                    .map(|(a, b)| a $op b)
                    .zip(result.iter_mut())
                    .for_each(|(val, out)| {
                        *out = val;
                    });

                result
            }
        }
    };
}

elementwise_unary_op!(Neg, neg, -);
elementwise_binary_op!(Add, add, +);
elementwise_binary_op!(Sub, sub, -);
elementwise_scalar_op!(Mul, mul, *);
elementwise_scalar_op!(Div, div, /);

impl<const N: usize> ops::Index<usize> for Vector<N> {
    type Output = i64;
    fn index(&self, index: usize) -> &i64 {
        &self.0[index]
    }
}

impl<const N: usize, const M: usize> ops::Index<(usize, usize)>
    for Matrix<N, M>
{
    type Output = i64;
    fn index(&self, index: (usize, usize)) -> &i64 {
        &self.0[index.0][index.1]
    }
}

impl<const N: usize, const R: usize, const M: usize> ops::Mul<Matrix<R, M>>
    for Matrix<N, R>
{
    type Output = Matrix<N, M>;
    fn mul(self, rhs: Matrix<R, M>) -> Self::Output {
        let mut values = [[0; M]; N];
        values.iter_mut().enumerate().for_each(|(i, row)| {
            row.iter_mut().enumerate().for_each(|(j, val)| {
                *val = (0..R).map(|k| self[(i, k)] * rhs[(k, j)]).sum();
            });
        });
        Matrix::<N, M>(values)
    }
}

impl<const N: usize, const M: usize> ops::Mul<Vector<M>> for Matrix<N, M> {
    type Output = Vector<N>;
    fn mul(self, rhs: Vector<M>) -> Vector<N> {
        let mut values = [0; N];
        values.iter_mut().enumerate().for_each(|(i, out)| {
            *out = (0..M).map(|j| self[(i, j)] * rhs[j]).sum();
        });
        Vector::<N>(values)
    }
}

impl<const N: usize> Vector<N> {
    pub fn new(arr: [i64; N]) -> Self {
        Self(arr)
    }

    pub fn zero() -> Self {
        Self([0; N])
    }

    pub fn iter(&self) -> impl Iterator<Item = i64> + '_ {
        self.0.iter().copied()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut i64> + '_ {
        self.0.iter_mut()
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

impl Vector<3> {
    pub fn x(&self) -> i64 {
        self.0[0]
    }

    pub fn y(&self) -> i64 {
        self.0[1]
    }

    pub fn z(&self) -> i64 {
        self.0[2]
    }
}

impl<const N: usize, const M: usize> Matrix<N, M> {
    pub fn new(values: [[i64; M]; N]) -> Self {
        Self(values)
    }

    pub fn zero() -> Self {
        Self([[0; M]; N])
    }

    pub fn iter_flat(&self) -> impl Iterator<Item = i64> + '_ {
        self.0.iter().flat_map(|row| row.iter()).copied()
    }

    pub fn iter_flat_mut(&mut self) -> impl Iterator<Item = &mut i64> + '_ {
        self.0.iter_mut().flat_map(|row| row.iter_mut())
    }
}

impl<const N: usize> Matrix<N, N> {
    pub fn identity() -> Self {
        let mut values = [[0; N]; N];

        values.iter_mut().enumerate().for_each(|(i, row)| {
            row.iter_mut()
                .enumerate()
                .filter(move |(j, _)| &i == j)
                .for_each(|(_, val)| {
                    *val = 1;
                })
        });

        Self(values)
    }

    pub fn pow(&self, power: usize) -> Self {
        (0..power).fold(Self::identity(), |cum_prod, _i| *self * cum_prod)
    }
}

impl Matrix<2, 2> {
    // 90 degree rotation about the origin, positive angle
    // (counter-clockwise).
    pub fn rotate() -> Self {
        Self([[0, -1], [1, 0]])
    }
}

impl Matrix<3, 3> {
    // 90 degree rotation about the x axis.
    pub fn rotate_x() -> Self {
        Self([[1, 0, 0], [0, 0, -1], [0, 1, 0]])
    }

    // 90 degree rotation about the y axis.
    pub fn rotate_y() -> Self {
        Self([[0, 0, -1], [0, 1, 0], [1, 0, 0]])
    }

    // 90 degree rotation about the z axis.
    pub fn rotate_z() -> Self {
        Self([[0, -1, 0], [1, 0, 0], [0, 0, 1]])
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
                Self::rotate_z().pow(beta)
                    * Self::rotate_x().pow(alpha)
                    * Self::rotate_z().pow(gamma)
            })
    }
}

impl<const N: usize> FromStr for Vector<N> {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let mut parsed_values = s.split(',').map(|s| s.parse::<i64>());

        let mut values = [0; N];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_parse() {
        let a = "1,2,3".parse::<Vector<3>>().unwrap();
        let b = Vector::new([1, 2, 3]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_vector_add() {
        let a = Vector::new([0, 1, 2, 3, 4, 5]);
        let b = Vector::new([10, 20, 30, 40, 50, 60]);
        let c = Vector::new([10, 21, 32, 43, 54, 65]);
        assert_eq!(a + b, c);
    }

    #[test]
    fn test_vector_sub() {
        let a = Vector::new([10, 20, 30, 40, 50, 60]);
        let b = Vector::new([0, 1, 2, 3, 4, 5]);
        let c = Vector::new([10, 19, 28, 37, 46, 55]);
        assert_eq!(a - b, c);
    }

    #[test]
    fn test_vector_neg() {
        let a = Vector::new([10, 20, 30, 40, 50, 60]);
        let b = Vector::new([-10, -20, -30, -40, -50, -60]);
        assert_eq!(-a, b);
    }

    #[test]
    fn test_vector_scale() {
        let a = Vector::new([10, 20, 30, 40, 50, 60]);
        let b = Vector::new([20, 40, 60, 80, 100, 120]);
        assert_eq!(a * 2, b);
    }

    #[test]
    fn test_vector_div() {
        let a = Vector::new([10, 20, 30, 40, 50, 60]);
        let b = Vector::new([5, 10, 15, 20, 25, 30]);
        assert_eq!(a / 2, b);
    }

    #[test]
    fn test_matrix_add() {
        let a = Matrix::new([[0, 1], [2, 3], [4, 5]]);
        let b = Matrix::new([[10, 20], [30, 40], [50, 60]]);
        let c = Matrix::new([[10, 21], [32, 43], [54, 65]]);
        assert_eq!(a + b, c);
    }

    #[test]
    fn test_matrix_sub() {
        let a = Matrix::new([[10, 20], [30, 40], [50, 60]]);
        let b = Matrix::new([[0, 1], [2, 3], [4, 5]]);
        let c = Matrix::new([[10, 19], [28, 37], [46, 55]]);
        assert_eq!(a - b, c);
    }

    #[test]
    fn test_matrix_neg() {
        let a = Matrix::new([[10, 20], [30, 40], [50, 60]]);
        let b = Matrix::new([[-10, -20], [-30, -40], [-50, -60]]);
        assert_eq!(-a, b);
    }

    #[test]
    fn test_matrix_scale() {
        let a = Matrix::new([[10, 20, 30], [40, 50, 60]]);
        let b = Matrix::new([[20, 40, 60], [80, 100, 120]]);
        assert_eq!(a * 2, b);
    }

    #[test]
    fn test_matrix_div() {
        let a = Matrix::new([[10, 20, 30], [40, 50, 60]]);
        let b = Matrix::new([[5, 10, 15], [20, 25, 30]]);
        assert_eq!(a / 2, b);
    }

    #[test]
    fn test_matrix_matrix_mul() {
        let a = Matrix::<1, 3>::new([[0, 1, 2]]);
        let b = Matrix::<3, 2>::new([[10, 20], [30, 40], [50, 60]]);
        let c = Matrix::<1, 2>::new([[
            10 * 0 + 30 * 1 + 50 * 2,
            20 * 0 + 40 * 1 + 60 * 2,
        ]]);
        println!("{:?} * {:?} = {:?}", a, b, a * b);
        assert_eq!(a * b, c);
    }

    #[test]
    fn test_matrix_vector_mul() {
        let a = Matrix::<3, 2>::new([[0, 1], [2, 3], [4, 5]]);
        let b = Vector::<2>::new([10, 20]);
        let c = Vector::<3>::new([
            10 * 0 + 20 * 1,
            10 * 2 + 20 * 3,
            10 * 4 + 20 * 5,
        ]);
        assert_eq!(a * b, c);
    }
}
