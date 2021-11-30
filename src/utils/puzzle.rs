use crate::utils::Error;

pub trait Puzzle {
    fn day(&self) -> i32;
    fn implemented(&self) -> bool;
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error>;
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error>;
}

impl dyn Puzzle {
    pub fn call_part(
        &self,
        part_num: i32,
    ) -> Result<Box<dyn std::fmt::Debug>, Error> {
        if part_num == 1 {
            self.part_1()
        } else if part_num == 2 {
            self.part_2()
        } else {
            Err(Error::InvalidArg(crate::utils::Arg::I32(part_num)))
        }
    }
}
