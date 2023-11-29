use std::cmp::PartialOrd;
use std::ops::RangeInclusive;

pub trait RangeIntersects<T> {
    fn intersects(&self, other: &T) -> bool;
}

impl<T1, T2> RangeIntersects<RangeInclusive<T1>> for RangeInclusive<T2>
where
    T1: PartialOrd<T2>,
    T2: PartialOrd<T1>,
{
    fn intersects(&self, other: &RangeInclusive<T1>) -> bool {
        (self.start() <= other.end()) && (other.start() <= self.end())
    }
}
