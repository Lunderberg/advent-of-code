use aoc_framework::Error;

use std::cmp;
use std::fmt::{Display, Formatter};
use std::ops;
use std::str::FromStr;

use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Vector<const N: usize, T = i64>([T; N]);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Matrix<const N: usize, const M: usize, T = i64>([Vector<M, T>; N]);

pub struct DisplayHelper<'a, T> {
    item: &'a T,
    line_prefix: Option<&'a str>,
    prefix_first_line: bool,
}

macro_rules! elementwise_unary_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<const N: usize, const M: usize, T> ops::$trait for Matrix<N, M, T>
        where T: ops::$trait<Output=T> {
            type Output = Self;
            fn $method(self) -> Self {
                Self(self.0.map(|row| row.map(|val| $op val)))
            }
        }

        impl<const N: usize, T> ops::$trait for Vector<N, T>
        where T: ops::$trait<Output=T> {
            type Output = Self;
            fn $method(self) -> Self {
                Self(self.0.map(|val| $op val))
            }
        }
    };
}

macro_rules! elementwise_scalar_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<const N: usize, const M: usize, T> ops::$trait<T> for Matrix<N, M, T>
        where T: ops::$trait<Output = T> + Copy {
            type Output = Self;
            fn $method(self, rhs: T) -> Self {
                Self(self.0.map(|row| row.map(|val| val $op rhs)))
            }
        }

        impl<const N: usize, T> ops::$trait<T> for Vector<N, T>
        where T: ops::$trait<Output = T> + Copy {
            type Output = Self;
            fn $method(self, rhs: T) -> Self {
                Self(self.0.map(|val| val $op rhs))
            }
        }

        // Would prefer to make a generic trait over the primitive
        // type.
        //
        //   impl<const N: usize, T> ops::$trait<Vector<N,T>> for T
        //   where T: ops::$trait<Output = T> + Copy {
        //       type Output = Vector<N, T>;
        //       fn $method(self, rhs: Vector<N,T>) -> Self::Output {
        //           rhs.0.map(|val| self $op val).into()
        //       }
        //   }
        //
        // However, that runs into E0210, since the implementation
        // over the generic type T isn't covered by any local type.
        // Therefore, implementing for several common types, and
        // crossing fingers that it doesn't become an issue later.
        elementwise_scalar_op_lhs!($trait, $method, $op, i8);
        elementwise_scalar_op_lhs!($trait, $method, $op, i16);
        elementwise_scalar_op_lhs!($trait, $method, $op, i32);
        elementwise_scalar_op_lhs!($trait, $method, $op, i64);
        elementwise_scalar_op_lhs!($trait, $method, $op, u8);
        elementwise_scalar_op_lhs!($trait, $method, $op, u16);
        elementwise_scalar_op_lhs!($trait, $method, $op, u32);
        elementwise_scalar_op_lhs!($trait, $method, $op, u64);
        elementwise_scalar_op_lhs!($trait, $method, $op, f32);
        elementwise_scalar_op_lhs!($trait, $method, $op, f64);
        elementwise_scalar_op_lhs!($trait, $method, $op, usize);
    };
}

macro_rules! elementwise_scalar_op_lhs {
    ($trait:ident, $method:ident, $op:tt, $prim:ident) => {
        impl<const N: usize> ops::$trait<Vector<N, $prim>> for $prim {
            type Output = Vector<N, $prim>;
            fn $method(self, rhs: Vector<N, $prim>) -> Self::Output {
                rhs.0.map(|val| self $op val).into()
            }
        }
    };
}

macro_rules! elementwise_binary_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<const N: usize, const M: usize, T> ops::$trait for Matrix<N, M, T>
        where T: ops::$trait<Output = T> + Copy {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self {
                Self::new(
                    std::array::from_fn(|i| self[i] $op rhs[i])
                )
            }
        }

        impl<const N: usize, T> ops::$trait for Vector<N, T>
        where T: ops::$trait<Output = T> + Copy  {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self {
                std::array::from_fn(|i| self[i] $op rhs[i]).into()
            }
        }
    };
}

macro_rules! elementwise_binary_assign_op {
    ($trait:ident, $method:ident, $op:tt) => {
        // TODO: Same overload for matrix

        impl<const N: usize, T, Rhs> ops::$trait<Vector<N,Rhs>> for Vector<N, T>
        where T: ops::$trait<Rhs> {
            fn $method(&mut self, rhs: Vector<N,Rhs>)  {
                self.iter_mut()
                    .zip(rhs.into_iter())
                    .for_each(|(a,b)|{
                        *a $op b;
                    });
            }
        }
    };
}

elementwise_unary_op!(Neg, neg, -);
elementwise_binary_assign_op!(AddAssign, add_assign, +=);
elementwise_binary_assign_op!(SubAssign, sub_assign, -=);
elementwise_binary_op!(Add, add, +);
elementwise_binary_op!(Sub, sub, -);
elementwise_scalar_op!(Mul, mul, *);
elementwise_scalar_op!(Div, div, /);

impl<const N: usize, T> ops::Index<usize> for Vector<N, T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl<const N: usize, T> ops::IndexMut<usize> for Vector<N, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<const N: usize, T> Display for Vector<N, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "(")?;
        self.iter().enumerate().try_for_each(|(i, val)| {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{val}")
        })?;
        write!(f, ")")?;
        Ok(())
    }
}
impl<'a, const M: usize, const N: usize, T> Display
    for DisplayHelper<'a, Matrix<N, M, T>>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let col_widths: [usize; M] = std::array::from_fn(|i| {
            (0..N)
                .map(|j| format!("{}", self.item[(j, i)]).len())
                .max()
                .unwrap_or(0)
        });
        let total_width = col_widths.iter().map(|w| w + 2).sum();
        let prefix = &self.line_prefix.unwrap_or("");

        if self.prefix_first_line {
            write!(f, "{prefix}")?;
        }

        writeln!(f, "┌{:width$}┐", "", width = total_width)?;
        self.item.iter_rows().try_for_each(|row| {
            write!(f, "{prefix}|")?;
            row.iter()
                .zip(col_widths.iter())
                .try_for_each(|(item, width)| write!(f, " {item:width$} "))?;
            writeln!(f, "|")
        })?;
        writeln!(f, "{prefix}└{:width$}┘", "", width = total_width)?;
        Ok(())
    }
}

impl<const N: usize, const M: usize, T> ops::Index<(usize, usize)>
    for Matrix<N, M, T>
{
    type Output = T;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.0[index.0][index.1]
    }
}

impl<const N: usize, const M: usize, T> ops::IndexMut<(usize, usize)>
    for Matrix<N, M, T>
{
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.0[index.0][index.1]
    }
}

impl<const N: usize, const M: usize, T> ops::Index<usize> for Matrix<N, M, T> {
    type Output = Vector<M, T>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const N: usize, const M: usize, T> ops::IndexMut<usize>
    for Matrix<N, M, T>
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<const N: usize, const R: usize, const M: usize, T>
    ops::Mul<Matrix<R, M, T>> for Matrix<N, R, T>
where
    T: Copy,
    T: ops::Mul<Output = T>,
    T: std::iter::Sum,
{
    type Output = Matrix<N, M, T>;
    fn mul(self, rhs: Matrix<R, M, T>) -> Self::Output {
        let values = std::array::from_fn(|i| {
            std::array::from_fn(|j| {
                (0..R).map(|k| self[(i, k)] * rhs[(k, j)]).sum()
            })
            .into()
        });
        Matrix::<N, M, T>(values)
    }
}

impl<const N: usize, const M: usize, T> ops::Mul<Vector<M, T>>
    for Matrix<N, M, T>
where
    T: Default + Copy + std::iter::Sum,
    T: ops::Mul<Output = T>,
{
    type Output = Vector<N, T>;
    fn mul(self, rhs: Vector<M, T>) -> Vector<N, T> {
        let mut values = [(); N].map(|_| T::default());
        values.iter_mut().enumerate().for_each(|(i, out)| {
            *out = (0..M).map(|j| self[(i, j)] * rhs[j]).sum();
        });
        values.into()
    }
}

impl<const N: usize, T> IntoIterator for Vector<N, T> {
    type Item = T;

    type IntoIter = <[T; N] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const N: usize, T> num::Zero for Vector<N, T>
where
    T: num::Zero,
    T: Copy,
{
    fn zero() -> Self {
        // Would be more succint to write as `[T::zero(), N]`, but
        // that would require `T: Copy`.  Since `T` is usually a
        // primitive, the likelihood of finding a `T` that implements
        // `num::Zero` but doesn't implement `Copy` seems pretty low,
        // but might as well avoid requiring it.
        std::array::from_fn(|_| T::zero()).into()
    }

    fn is_zero(&self) -> bool {
        self.iter().all(|value| value.is_zero())
    }
}

impl<const N: usize, T> Vector<N, T> {
    pub fn new(arr: [T; N]) -> Self {
        Self(arr)
    }

    pub fn zero() -> Self
    where
        T: num::Zero,
    {
        Self([(); N].map(|_| T::zero()))
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.0.iter_mut()
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.0.swap(a, b)
    }

    pub fn dist2(&self, other: &Self) -> T
    where
        T: ops::Sub<Output = T> + ops::Mul<Output = T>,
        T: std::iter::Sum + Copy,
    {
        self.iter()
            .zip(other.iter())
            .map(|(a, b)| *a - *b)
            .map(|diff| diff * diff)
            .sum()
    }

    pub fn manhattan_dist(&self, other: &Self) -> T
    where
        T: cmp::PartialOrd + ops::Sub<Output = T>,
        T: Copy + std::iter::Sum,
    {
        self.iter()
            .zip(other.iter())
            .map(|(a, b)| if a < b { *b - *a } else { *a - *b })
            .sum()
    }

    pub fn map<U, F>(self, func: F) -> Vector<N, U>
    where
        F: FnMut(T) -> U,
    {
        Vector(self.0.map(func))
    }

    pub fn zip_map<F>(self, other: Self, mut func: F) -> Self
    where
        F: FnMut(&T, &T) -> T,
        T: num::Zero,
    {
        let mut result = Self::zero();
        self.iter()
            .zip(other.iter())
            .map(|(a, b)| func(a, b))
            .zip(result.iter_mut())
            .for_each(|(val, out)| {
                *out = val;
            });
        result
    }

    /// Points from self to other, including both endpoints.  Assumes
    /// that only one coordinate differs between self and other.
    pub fn cardinal_points_to(
        &self,
        other: &Self,
    ) -> impl Iterator<Item = Vector<N, T>> + '_
    where
        T: Copy + std::iter::Sum,
        T: ops::Add<Output = T> + ops::Sub<Output = T> + ops::Mul<Output = T>,
        T: num::Integer + num::Signed + num::Zero + num::One,
        T: num::ToPrimitive,
    {
        let delta: Self = *other - *self;
        let step: Self = delta.map(|val| num::signum(val));
        let len: T = delta.manhattan_dist(&Vector::zero());
        num::range_inclusive(T::zero(), len).map(move |i: T| -> Vector<N, T> {
            let offset: Vector<N, T> = step * i;
            *self + offset
        })
        // std::iter::successors(Some(*self), move |&prev| Some(prev + step))
        //     .take((len + T::one()).into())
    }

    pub fn dot_product(self, other: Self) -> T
    where
        T: std::iter::Sum,
        T: num::Zero,
        T: std::ops::Mul<T, Output = T>,
    {
        self.into_iter()
            .zip(other.into_iter())
            .map(|(a, b)| a * b)
            .sum()
    }
}

impl<const N: usize, T> Default for Vector<N, T>
where
    T: Default,
{
    fn default() -> Self {
        [(); N].map(|_| T::default()).into()
    }
}

impl<T> Vector<2, T> {
    pub fn x(&self) -> T
    where
        T: Copy,
    {
        self.0[0]
    }

    pub fn y(&self) -> T
    where
        T: Copy,
    {
        self.0[1]
    }
}

impl<T> Vector<3, T> {
    pub fn x(&self) -> T
    where
        T: Copy,
    {
        self.0[0]
    }

    pub fn y(&self) -> T
    where
        T: Copy,
    {
        self.0[1]
    }

    pub fn z(&self) -> T
    where
        T: Copy,
    {
        self.0[2]
    }
}

impl<const N: usize, T> From<[T; N]> for Vector<N, T> {
    fn from(values: [T; N]) -> Self {
        Self::new(values)
    }
}

impl<const N: usize, T> From<Vector<N, T>> for [T; N] {
    fn from(value: Vector<N, T>) -> Self {
        value.0
    }
}

// TODO: Macro for generating these converters for more tuple sizes.
impl<T> From<Vector<2, T>> for (T, T) {
    fn from(value: Vector<2, T>) -> Self {
        IntoIterator::into_iter(value.0).collect_tuple().unwrap()
    }
}

impl<T, A, B> From<(A, B)> for Vector<2, T>
where
    T: From<A>,
    T: From<B>,
{
    fn from(value: (A, B)) -> Self {
        [value.0.into(), value.1.into()].into()
    }
}

impl<const N: usize, const M: usize, T> num::Zero for Matrix<N, M, T>
where
    T: num::Zero,
    T: Copy,
{
    fn zero() -> Self {
        Matrix::new(std::array::from_fn(|_| Vector::zero()))
    }

    fn is_zero(&self) -> bool {
        self.iter_rows().all(|row| row.is_zero())
    }
}

impl<const N: usize, const M: usize, T> Matrix<N, M, T> {
    pub fn new<Row>(rows: [Row; N]) -> Self
    where
        Row: Into<Vector<M, T>>,
    {
        Self(rows.map(|row| row.into()))
    }

    pub fn transpose(self) -> Matrix<M, N, T> {
        let mut take_from = self.0.map(|row| row.map(|item| Some(item)));

        Matrix::new(std::array::from_fn(|i| {
            std::array::from_fn(|j| {
                take_from[j][i].take().expect(
                    "Internal error, \
                     transpose should only take \
                     each element once.",
                )
            })
        }))
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = &Vector<M, T>> + '_ {
        self.0.iter()
    }

    pub fn swap_rows(&mut self, a: usize, b: usize) {
        self.0.swap(a, b)
    }

    pub fn iter_flat(&self) -> impl Iterator<Item = &T> + '_ {
        self.0.iter().flat_map(|row| row.iter())
    }

    pub fn iter_flat_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.0.iter_mut().flat_map(|row| row.iter_mut())
    }

    pub fn display(&self) -> DisplayHelper<'_, Self> {
        DisplayHelper {
            item: self,
            line_prefix: None,
            prefix_first_line: false,
        }
    }
}

impl<'a, T> DisplayHelper<'a, T> {
    pub fn line_prefix<'b>(self, line_prefix: &'b str) -> DisplayHelper<'b, T>
    where
        'a: 'b,
    {
        DisplayHelper {
            line_prefix: Some(line_prefix),
            ..self
        }
    }

    pub fn prefix_first_line(self, prefix_first_line: bool) -> Self {
        Self {
            prefix_first_line,
            ..self
        }
    }
}

impl<const N: usize, T> Matrix<N, N, T> {
    pub fn identity() -> Self
    where
        T: num::Zero + num::One,
    {
        Matrix::new(std::array::from_fn(|i| {
            std::array::from_fn(|j| if i == j { T::one() } else { T::zero() })
        }))
    }

    pub fn pow(&self, power: usize) -> Self
    where
        T: Copy,
        T: num::Zero + num::One,
        T: ops::Mul<Output = T>,
        T: std::iter::Sum,
    {
        (0..power).fold(Self::identity(), |cum_prod, _i| *self * cum_prod)
    }
}

impl<T> Matrix<2, 2, T> {
    // 90 degree rotation about the origin, positive angle
    // (counter-clockwise).
    pub fn rotate() -> Self
    where
        T: num::Zero + num::One + num::Signed,
    {
        Self::new([[T::zero(), T::zero() - T::one()], [T::one(), T::zero()]])
    }
}

impl Matrix<3, 3> {
    // 90 degree rotation about the x axis.
    pub fn rotate_x() -> Self {
        Self::new([[1, 0, 0], [0, 0, -1], [0, 1, 0]])
    }

    // 90 degree rotation about the y axis.
    pub fn rotate_y() -> Self {
        Self::new([[0, 0, -1], [0, 1, 0], [1, 0, 0]])
    }

    // 90 degree rotation about the z axis.
    pub fn rotate_z() -> Self {
        Self::new([[0, -1, 0], [1, 0, 0], [0, 0, 1]])
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

impl<const N: usize, T> FromStr for Vector<N, T>
where
    T: Default,
    T: FromStr,
    Error: From<T::Err>,
{
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let mut parsed_values = s.split(',').map(|s| s.parse::<T>());

        let mut values = [(); N].map(|_| T::default());
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
        let c = Matrix::<1, 2>::new([[30 + 50 * 2, 40 + 60 * 2]]);
        println!("{:?} * {:?} = {:?}", a, b, a * b);
        assert_eq!(a * b, c);
    }

    #[test]
    fn test_matrix_vector_mul() {
        let a = Matrix::<3, 2>::new([[0, 1], [2, 3], [4, 5]]);
        let b = Vector::<2>::new([10, 20]);
        let c = Vector::<3>::new([20, 10 * 2 + 20 * 3, 10 * 4 + 20 * 5]);
        assert_eq!(a * b, c);
    }
}
