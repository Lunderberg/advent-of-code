use std::cmp::PartialOrd;
use std::ops::{Range, RangeInclusive};

pub trait RangeIntersection: Sized {
    fn intersection(&self, other: &Self) -> Option<Self>;
}

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

impl<T> RangeIntersection for RangeInclusive<T>
where
    T: Sized,
    T: Ord,
    T: Copy,
{
    fn intersection(&self, other: &Self) -> Option<Self> {
        if self.start() <= other.end() && other.start() <= self.end() {
            let start = std::cmp::max(*self.start(), *other.start());
            let end = std::cmp::min(*self.end(), *other.end());
            Some(start..=end)
        } else {
            None
        }
    }
}

impl<T1, T2> RangeIntersects<Range<T1>> for Range<T2>
where
    T1: PartialOrd<T2>,
    T2: PartialOrd<T1>,
{
    fn intersects(&self, other: &Range<T1>) -> bool {
        (self.start < other.end) && (other.start < self.end)
    }
}

impl<T> RangeIntersection for Range<T>
where
    T: Ord,
    T: Copy,
{
    fn intersection(&self, other: &Self) -> Option<Self> {
        if self.start < other.end && other.start < self.end {
            let start = std::cmp::max(self.start, other.start);
            let end = std::cmp::min(self.end, other.end);
            Some(start..end)
        } else {
            None
        }
    }
}
