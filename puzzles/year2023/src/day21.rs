use std::collections::HashMap;
use std::fmt::Display;

use aoc_utils::direction::Direction;
use aoc_utils::prelude::*;

pub struct GardenMap {
    map: GridMap<Tile>,
    tiled: bool,
}

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
        write!(f, "{}", self.map)
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
        self.map
            .iter()
            .find(|(_, tile): &(_, &_)| tile.is_elf())
            .map(|(pos, _)| pos)
    }

    fn apply_map_tile(
        &self,
        global_pos: Vector<2, i64>,
    ) -> (Vector<2, i64>, Vector<2, i64>) {
        let (width, height) = self.map.shape();
        let width = width as i64;
        let height = height as i64;

        let tiled_map = [
            global_pos.x().div_euclid(width),
            global_pos.y().div_euclid(height),
        ]
        .into();

        let local_pos = [
            global_pos.x().rem_euclid(width),
            global_pos.y().rem_euclid(height),
        ]
        .into();

        (tiled_map, local_pos)
    }

    fn is_garden_tile(&self, pos: Vector<2, i64>) -> bool {
        let local_pos = if self.tiled {
            self.apply_map_tile(pos).1
        } else {
            pos
        };

        self.map
            .get(local_pos)
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
        let map = lines.collect();
        Ok(GardenMap { map, tiled: false })
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
        let num_steps: u64 = 26501365;

        // Verifying a few properties of the user-specific maps before
        // taking advantage of them.

        // 1. The garden is a square with an odd number of tiles.
        let shape = garden.map.shape_vec();
        assert_eq!(shape.x(), shape.y());
        assert!(shape.x() % 2 == 1);

        // 2. The elf starts at the center of the map.
        let initial = garden.elf_location().unwrap();
        assert_eq!(initial * 2 + [1, 1].into(), shape);

        // 3. Neither the row nor the column on which the elf starts
        // contain a rock.
        assert!(std::iter::empty()
            .chain((0..shape.x()).map(|x| garden.map[(x, initial.y())]))
            .chain((0..shape.y()).map(|y| garden.map[(initial.x(), y)]))
            .all(|tile| tile.is_garden()));

        // 4. The perimeter doesn't contain any rocks.
        assert!(std::iter::empty()
            .chain((0..shape.x()).map(|x| garden.map[(x, 0)]))
            .chain((0..shape.y()).map(|y| garden.map[(0, y)]))
            .chain((0..shape.x()).map(|x| garden.map[(x, shape.y() - 1)]))
            .chain((0..shape.y()).map(|y| garden.map[(shape.x() - 1, y)]))
            .all(|tile| tile.is_garden()));

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
                        .chain(garden.map.iter().map(|pos| (pos, None)))
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

        let partially_visited_tile_radius =
            (num_steps + 1).div_ceil(shape.x() as u64) + 1;

        // This is still inefficient to a degree, since all tiles up
        // through some the last 2-3 will be fully visited, and could
        // be determined explicitly.  However, that would be another
        // possibility for off-by-one errors, and this is fast enough
        // for the time being.
        let reachable_garden_tiles = (0..partially_visited_tile_radius)
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
                let steps_to_corner =
                    i.saturating_sub(1) * (shape.x() as u64) + 1;
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
                    let tiles_per_map = if starting_step_count
                        + max_steps_in_map
                        <= num_steps
                    {
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
                            .filter(|&steps| steps <= num_steps)
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
