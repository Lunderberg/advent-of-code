pub trait Puzzle {
    fn day(&self) -> i32;
    fn implemented(&self) -> bool;
    fn part_1(&self) -> ();
    fn part_2(&self) -> ();
}
