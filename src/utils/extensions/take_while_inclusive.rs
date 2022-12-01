pub trait TakeWhileInclusive: Iterator {
    fn take_while_inclusive<P>(
        &mut self,
        predicate: P,
    ) -> TakeWhileInclusiveIter<'_, Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        TakeWhileInclusiveIter::new(self, predicate)
    }
}

impl<T> TakeWhileInclusive for T where T: Iterator {}

pub struct TakeWhileInclusiveIter<'a, I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    iter: &'a mut I,
    predicate: P,
    prev_predicate: bool,
}

impl<'a, I, P> TakeWhileInclusiveIter<'a, I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    fn new(iter: &'a mut I, predicate: P) -> Self {
        Self {
            iter,
            predicate,
            prev_predicate: true,
        }
    }
}

impl<I, P> Iterator for TakeWhileInclusiveIter<'_, I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        (self.prev_predicate)
            .then(|| {
                let opt_item = self.iter.next();
                if let Some(ref item) = opt_item {
                    self.prev_predicate = (self.predicate)(item);
                }
                opt_item
            })
            .flatten()
    }
}
