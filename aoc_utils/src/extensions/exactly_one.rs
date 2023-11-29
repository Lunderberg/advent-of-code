use aoc_framework::Error;
use itertools::Itertools;

pub trait ExactlyOneExt: Iterator {
    fn exactly_one_or_err(self) -> Result<Self::Item, Error>;
}

impl<Iter: Iterator> ExactlyOneExt for Iter {
    fn exactly_one_or_err(self) -> Result<Self::Item, Error> {
        self.exactly_one().map_err(|_| Error::ExpectedExactlyOne)
    }
}
