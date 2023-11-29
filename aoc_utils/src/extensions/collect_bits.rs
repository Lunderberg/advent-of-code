pub trait CollectBits: Iterator<Item = bool> {
    fn collect_bits<V>(self) -> V
    where
        Self: Sized,
        V: num::PrimInt,
        V: From<bool>,
    {
        self.fold(V::zero(), |acc, b| (acc << 1) + b.into())
    }
}

impl<T> CollectBits for T where T: Iterator<Item = bool> {}
