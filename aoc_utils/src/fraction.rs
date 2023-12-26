use std::fmt::Display;
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Sub};

use num::integer::gcd as find_gcd;

#[derive(Debug, Clone, Copy)]
pub struct Fraction<T = i64> {
    pub num: T,
    pub denom: T,
}

impl<T> From<T> for Fraction<T>
where
    T: num::One,
{
    fn from(value: T) -> Self {
        Self {
            num: value,
            denom: T::one(),
        }
    }
}

impl Into<f64> for Fraction {
    fn into(self) -> f64 {
        (self.num as f64) / (self.denom as f64)
    }
}

impl<T> Display for Fraction<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.num, self.denom)
    }
}

impl<T> PartialEq for Fraction<T>
where
    T: Copy,
    T: PartialEq,
    T: Mul<Output = T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.num * other.denom == other.num * self.denom
    }
}
impl<T> Eq for Fraction<T>
where
    T: Copy,
    T: PartialEq,
    T: Mul<Output = T>,
{
}

impl<T> PartialOrd for Fraction<T>
where
    T: Copy,
    T: Ord,
    T: Mul<Output = T>,
    T: num::Zero,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for Fraction<T>
where
    T: Copy,
    T: Ord,
    T: Mul<Output = T>,
    T: num::Zero,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // a/b < c/d
        // a*d < b*c (reverse based on sign of b*d)

        match (self.denom.cmp(&T::zero()), other.denom.cmp(&T::zero())) {
            (std::cmp::Ordering::Equal, std::cmp::Ordering::Less) => todo!(),
            (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal) => todo!(),
            (std::cmp::Ordering::Equal, std::cmp::Ordering::Greater) => todo!(),
            (std::cmp::Ordering::Less, std::cmp::Ordering::Equal) => todo!(),
            (std::cmp::Ordering::Greater, std::cmp::Ordering::Equal) => todo!(),

            (std::cmp::Ordering::Less, std::cmp::Ordering::Less)
            | (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => {
                (self.num * other.denom).cmp(&(other.num * self.denom))
            }

            (std::cmp::Ordering::Less, std::cmp::Ordering::Greater)
            | (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => {
                (other.num * self.denom).cmp(&(self.num * other.denom))
            }
        }
    }
}

impl<T> Fraction<T> {
    pub fn normalize(self) -> Self
    where
        T: Copy,
        T: num::Integer,
        T: num::Zero,
        T: PartialOrd,
        T: Sub<Output = T>,
        T: Div<Output = T>,
    {
        let Self { num, denom } = self;

        let (num, denom) = if denom < T::zero() {
            (T::zero() - num, T::zero() - denom)
        } else {
            (num, denom)
        };
        let gcd = find_gcd(num, denom);
        let (num, denom) = (num / gcd, denom / gcd);

        Self { num, denom }
    }
}

impl<T> PartialEq<T> for Fraction<T>
where
    T: Mul<Output = T>,
    T: PartialEq,
    T: Copy,
{
    fn eq(&self, other: &T) -> bool {
        self.num == self.denom * *other
    }
}

impl<T> Add for Fraction<T>
where
    T: num::Integer,
    T: Copy,
    T: Add<Output = T>,
    T: Mul<Output = T>,
    T: Div<Output = T>,
{
    type Output = Fraction<T>;

    fn add(self, rhs: Self) -> Self::Output {
        let gcd: T = find_gcd(self.denom, rhs.denom);
        let num = self.num * (rhs.denom / gcd) + rhs.num * (self.denom / gcd);
        let denom = self.denom * rhs.denom;
        Self { num, denom }
    }
}

impl<T> Sub for Fraction<T>
where
    T: num::Integer,
    T: Copy,
    T: Sub<Output = T>,
    T: Mul<Output = T>,
    T: Div<Output = T>,
{
    type Output = Fraction<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        let gcd: T = find_gcd(self.denom, rhs.denom);
        let num = self.num * (rhs.denom / gcd) - rhs.num * (self.denom / gcd);
        let denom = self.denom * rhs.denom;
        Self { num, denom }
    }
}

impl<T> Div for Fraction<T>
where
    T: Mul<Output = T>,
{
    type Output = Fraction<T>;

    fn div(self, rhs: Self) -> Self::Output {
        let num = self.num * rhs.denom;
        let denom = self.denom * rhs.num;
        Self { num, denom }
    }
}

impl<T> Mul for Fraction<T>
where
    T: Mul<Output = T>,
{
    type Output = Fraction<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        let num = self.num * rhs.num;
        let denom = self.denom * rhs.denom;
        Self { num, denom }
    }
}

impl<T> Sum for Fraction<T>
where
    T: num::Integer,
    T: Copy,
    T: num::Zero + num::One,
    T: Div<Output = T>,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(
            Fraction {
                num: T::zero(),
                denom: T::one(),
            },
            |a, b| a + b,
        )
    }
}

impl<T> num::Zero for Fraction<T>
where
    T: num::Integer,
    T: Copy,
    T: num::Zero + num::One,
    T: Div<Output = T>,
{
    fn zero() -> Self {
        Fraction {
            num: T::zero(),
            denom: T::one(),
        }
    }

    fn is_zero(&self) -> bool {
        self.num.is_zero()
    }
}

impl<T> num::One for Fraction<T>
where
    T: num::One,
{
    fn one() -> Self {
        Fraction {
            num: T::one(),
            denom: T::one(),
        }
    }
}
