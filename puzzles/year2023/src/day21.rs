use std::collections::HashMap;
use std::{collections::HashSet, fmt::Display};

use aoc_utils::direction::Direction;
use aoc_utils::{prelude::*, IntoGridPos};

use indicatif::ProgressIterator;

pub struct GardenMap(GridMap<Tile>);

#[derive(Debug, Clone, Copy)]
enum Tile {
    Rock,
    Garden,
    Elf,
}

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::Garden),
            '#' => Ok(Tile::Rock),
            'S' => Ok(Tile::Elf),
            _ => Err(Error::UnknownChar(c)),
        }
    }
}
impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Tile::Rock => '#',
            Tile::Garden => '.',
            Tile::Elf => 'S',
        };
        write!(f, "{c}")
    }
}
impl Display for GardenMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Tile {
    fn is_garden(&self) -> bool {
        matches!(self, Tile::Garden | Tile::Elf)
    }
    fn is_elf(&self) -> bool {
        matches!(self, Tile::Elf)
    }
}

impl GardenMap {
    fn elf_location(&self) -> Option<Vector<2, i64>> {
        self.0
            .iter()
            .find(|(_, tile): &(_, &_)| tile.is_elf())
            .map(|(pos, _)| pos)
    }

    fn without_elf(&self) -> Self {
        let map = self
            .0
            .iter()
            .map(|(pos, &tile)| {
                (pos, if tile.is_elf() { Tile::Garden } else { tile })
            })
            .collect();
        Self(map)
    }

    fn with_elves(&self, elves: &HashSet<Vector<2, i64>>) -> Self {
        let map = self
            .0
            .iter()
            .map(|(pos, &tile)| {
                (
                    pos,
                    if elves.contains(&pos) {
                        Tile::Elf
                    } else if tile.is_garden() {
                        Tile::Garden
                    } else {
                        tile
                    },
                )
            })
            .collect();
        Self(map)
    }

    fn is_garden_tile(&self, pos: impl IntoGridPos) -> bool {
        self.0
            .get(pos)
            .map(|tile| tile.is_garden())
            .unwrap_or(false)
    }
}

impl EdgeWeightedGraph<Vector<2, i64>> for GardenMap {
    fn connections_from<'a>(
        &'a self,
        old_pos: &'a Vector<2, i64>,
    ) -> impl Iterator<Item = (Vector<2, i64>, u64)> + '_ {
        Direction::iter_cardinal()
            .map(move |dir| *old_pos + dir.as_vec())
            .filter(|&new_pos| self.is_garden_tile(new_pos))
            .map(|new_pos| (new_pos, 1))
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = GardenMap;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(GardenMap(lines.collect()))
    }

    fn part_1(
        garden: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let initial = garden.elf_location().unwrap();
        let num_steps = 64;

        let num_final_locations = garden
            .iter_dijkstra([initial])
            .take_while(|search_item| search_item.total_dist <= num_steps)
            .filter(|search_item| search_item.total_dist % 2 == num_steps % 2)
            .count();

        Ok(num_final_locations)
    }

    fn part_2(
        garden: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        // 600089785143895: Too low
        // 600090464062431: 5 minute delay
        // 600090570270195: Too high

        let num_steps: u64 = 26501365;

        // Verifying a few properties of the user-specific maps before
        // taking advantage of them.

        // 1. The garden is a square with an odd number of tiles.
        let shape = garden.0.shape_vec();
        assert_eq!(shape.x(), shape.y());
        assert!(shape.x() % 2 == 1);

        // 2. The elf starts at the center of the map.
        let initial = garden.elf_location().unwrap();
        assert_eq!(initial * 2 + [1, 1].into(), shape);

        // 3. Neither the row nor the column on which the elf starts
        // contain a rock.
        assert!(std::iter::empty()
            .chain((0..shape.x()).map(|x| garden.0[(x, initial.y())]))
            .chain((0..shape.y()).map(|y| garden.0[(initial.x(), y)]))
            .all(|tile| tile.is_garden()));

        // 4. The perimeter doesn't contain any rocks.
        assert!(std::iter::empty()
            .chain((0..shape.x()).map(|x| garden.0[(x, 0)]))
            .chain((0..shape.y()).map(|y| garden.0[(0, y)]))
            .chain((0..shape.x()).map(|x| garden.0[(x, shape.y() - 1)]))
            .chain((0..shape.y()).map(|y| garden.0[(shape.x() - 1, y)]))
            .all(|tile| tile.is_garden()));

        println!("The assertions hold");

        // With those assertions out of the way (which only hold for
        // the puzzle input, not for the example input), we can make
        // use of the following simplifying assumptions:
        //
        // 1. The elf can reach every repeated instance of the map.
        //
        // 2. The fastest path to a specific instance of the map is to
        // go directly along a cardinal direction.  There are no
        // shortcuts.
        //
        // 3. All directions are equally fast to travel in, on scales
        // larger than the map.
        //
        // 4. There are only 9 types of maps that must be considered.
        //    a. (4) Entering the map at one of the four corners.
        //    b. (4) Entering the map at the center of one of the
        //       four sides.
        //    c. (1) Starting the map at the center.

        println!("Shape = {shape}");

        let categories: HashMap<Vector<2, i64>, GridMap<Option<u64>>> =
            [0, initial.x(), shape.x() - 1]
                .into_iter()
                .cartesian_product([0, initial.y(), shape.y() - 1].into_iter())
                .map(|(x, y)| -> Vector<2, i64> { [x, y].into() })
                .map(|enter_at| -> (Vector<2, i64>, GridMap<Option<u64>>) {
                    let step_counts = garden
                        .iter_dijkstra([enter_at])
                        .map(|search_item| {
                            (
                                search_item.item,
                                Some(search_item.total_dist as u64),
                            )
                        })
                        .chain(garden.0.iter().map(|pos| (pos, None)))
                        .unique_by(|(pos, _)| *pos)
                        .collect();
                    (enter_at, step_counts)
                })
                .collect();

        let fully_visiting_maps: HashMap<Vector<2, i64>, (u64, u64)> =
            categories
                .iter()
                .map(|(enter_at, step_counts)| {
                    let (spaces_even, spaces_odd) = step_counts
                        .iter()
                        .filter_map(|steps: &Option<u64>| steps.as_ref())
                        .fold((0, 0), |(a, b), steps| {
                            if steps % 2 == 0 {
                                (a + 1, b)
                            } else {
                                (a, b + 1)
                            }
                        });
                    (*enter_at, (spaces_even, spaces_odd))
                })
                .collect();

        // categories
        //     .iter()
        //     .sorted_by_key(|(enter_at, _)| -> (i64, i64) {
        //         (enter_at.x(), enter_at.y())
        //     })
        //     .for_each(|(enter_at, step_counts)| {
        //         let step_digits: GridMap<_> = step_counts
        //             .iter()
        //             .map(|(pos, opt_steps): (Vector<2, i64>, &_)| {
        //                 (
        //                     pos,
        //                     opt_steps
        //                         .map(|steps| {
        //                             char::from_digit((steps % 10) as u32, 10)
        //                                 .unwrap()
        //                         })
        //                         .unwrap_or('#'),
        //                 )
        //             })
        //             .collect();
        //         println!("Enter at: {enter_at}\n{step_digits}");
        //     });

        // Used to determine how many maps around the outer border
        // need to be inspected.  For my input, it ended up being the
        // diagonal (`shape.x() + shape.y() - 2`), but there could be
        // a spiral that takes longer to traverse than to cross over.
        let max_steps_in_map = categories
            .iter()
            .flat_map(|(_, step_counts)| step_counts.iter())
            .filter_map(|steps: &Option<u64>| steps.clone())
            .max()
            .unwrap();

        let tile_x_min =
            (initial.x() - (num_steps as i64)).div_euclid(shape.x());
        let tile_x_max =
            (initial.x() + (num_steps as i64)).div_euclid(shape.x());
        let tile_y_min =
            (initial.y() - (num_steps as i64)).div_euclid(shape.y());
        let tile_y_max =
            (initial.y() + (num_steps as i64)).div_euclid(shape.y());

        let fully_visited_tile_radius =
            (num_steps - max_steps_in_map).div_euclid(shape.x() as u64);
        let partially_visited_tile_radius =
            (num_steps).div_ceil(shape.x() as u64) + 10;

        println!("Max steps in map: {max_steps_in_map}");
        println!("X tiles; {tile_x_min} through {tile_x_max} (inclusive)");
        println!("Y tiles; {tile_y_min} through {tile_y_max} (inclusive)");
        println!("Fully visited radius: {fully_visited_tile_radius}");
        println!("Partially visited radius: {partially_visited_tile_radius}");

        // The initial starting point
        let from_center_on_even_step = 1;

        // The distance
        let cardinal_side = fully_visited_tile_radius - 1;
        let from_side_on_odd_step =
            cardinal_side.div_euclid(2) + (initial.x() as u64) % 2;
        let from_side_on_even_step =
            cardinal_side.div_euclid(2) + (initial.x() as u64 + 1) % 2;

        // Distance=1 from center of (0,0) to top-right, then another
        // distance = 1 to offset.
        let triangle_side = fully_visited_tile_radius - 2;
        // Number of map iterations in the triangle, entered on an
        // even number of steps.  Take the side of the triangle, and
        // divide by two to find the number of odd numbers present.
        // Then, square it to find the sum of the first N odd numbers.
        let from_corner_on_even_step = (triangle_side + 1).div_euclid(2).pow(2);
        let from_corner_on_odd_step =
            triangle_side * (triangle_side + 1) / 2 - from_corner_on_even_step;

        println!("from_side_on_odd_step = {from_side_on_odd_step}");
        println!("from_side_on_even_step = {from_side_on_even_step}");
        println!("from_corner_on_odd_step = {from_corner_on_odd_step}");
        println!("from_corner_on_even_step = {from_corner_on_even_step}");

        let reachable_garden_tiles = (0..partially_visited_tile_radius)
            .progress_count(partially_visited_tile_radius)
            .flat_map(|i| {
                // Initial tile, for i==0
                let starting_tile =
                    std::iter::once((initial, if i == 0 { 1 } else { 0 }, 0));

                // On cardinal directions, for i>0.
                let steps_to_side = i.saturating_sub(1) * (shape.x() as u64)
                    + (initial.x() as u64)
                    + 1;
                let sides = [
                    (initial.x(), 0).into(),
                    (0, initial.y()).into(),
                    (initial.x(), shape.y() - 1).into(),
                    (shape.x() - 1, initial.y()).into(),
                ]
                .into_iter()
                .map(move |pos| {
                    (pos, if i == 0 { 0 } else { 1 }, steps_to_side)
                });

                // Entering from corner,
                let steps_to_corner = i.saturating_sub(1) * (shape.x() as u64);
                let corners = [
                    (0, 0).into(),
                    (0, shape.y() - 1).into(),
                    (shape.x() - 1, 0).into(),
                    (shape.x() - 1, shape.y() - 1).into(),
                ]
                .into_iter()
                .map(move |pos| (pos, i.saturating_sub(1), steps_to_corner));

                std::iter::empty()
                    .chain(starting_tile)
                    .chain(sides)
                    .chain(corners)
            })
            .filter(|(_, num_maps, _)| *num_maps > 0)
            .map(
                |(starting_loc, num_maps, starting_step_count): (
                    Vector<2, i64>,
                    u64,
                    u64,
                )| {
                    let tiles_per_map =
                        if starting_step_count + max_steps_in_map < num_steps {
                            let (with_even_steps, with_odd_steps) =
                                fully_visiting_maps[&starting_loc];
                            if starting_step_count % 2 == num_steps % 2 {
                                with_even_steps
                            } else {
                                with_odd_steps
                            }
                        } else {
                            categories[&starting_loc]
                                .iter()
                                .filter_map(|steps: &Option<u64>| {
                                    steps.map(|s| s + starting_step_count)
                                })
                                .filter(|&steps| steps < num_steps)
                                .filter(|&steps| steps % 2 == num_steps % 2)
                                .count() as u64
                        };
                    num_maps * tiles_per_map
                },
            )
            .sum::<u64>();

        Ok(reachable_garden_tiles)
    }
}
