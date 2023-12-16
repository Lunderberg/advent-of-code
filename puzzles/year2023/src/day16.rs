use std::fmt::Display;

use aoc_utils::prelude::*;

pub struct BeamMap(GridMap<Tile>);

#[derive(Debug, Clone, Copy)]
enum Tile {
    Empty,
    MirrorLeft,
    MirrorRight,
    SplitterHorizontal,
    SplitterVertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::Empty),
            '-' => Ok(Tile::SplitterHorizontal),
            '|' => Ok(Tile::SplitterVertical),
            '/' => Ok(Tile::MirrorLeft),
            '\\' => Ok(Tile::MirrorRight),
            _ => Err(Error::UnknownChar(c)),
        }
    }
}
impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Tile::Empty => '.',
            Tile::MirrorLeft => '/',
            Tile::MirrorRight => '\\',
            Tile::SplitterHorizontal => '-',
            Tile::SplitterVertical => '|',
        };
        write!(f, "{c}")
    }
}
impl Display for BeamMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Direction {
    fn as_vec(self) -> Vector<2, i64> {
        match self {
            Direction::Up => [0, -1].into(),
            Direction::Down => [0, 1].into(),
            Direction::Left => [-1, 0].into(),
            Direction::Right => [1, 0].into(),
        }
    }
}

impl Tile {
    fn outgoing_direction(
        self,
        dir: Direction,
    ) -> impl Iterator<Item = Direction> {
        let (primary, split) = match (self, dir) {
            (Tile::MirrorLeft, Direction::Up) => (Direction::Right, None),
            (Tile::MirrorLeft, Direction::Down) => (Direction::Left, None),
            (Tile::MirrorLeft, Direction::Left) => (Direction::Down, None),
            (Tile::MirrorLeft, Direction::Right) => (Direction::Up, None),

            (Tile::MirrorRight, Direction::Up) => (Direction::Left, None),
            (Tile::MirrorRight, Direction::Down) => (Direction::Right, None),
            (Tile::MirrorRight, Direction::Left) => (Direction::Up, None),
            (Tile::MirrorRight, Direction::Right) => (Direction::Down, None),

            (Tile::SplitterHorizontal, Direction::Up | Direction::Down) => {
                (Direction::Left, Some(Direction::Right))
            }
            (Tile::SplitterVertical, Direction::Left | Direction::Right) => {
                (Direction::Up, Some(Direction::Down))
            }

            _ => (dir, None),
        };
        std::iter::once(primary).chain(split)
    }
}

impl BeamMap {
    fn initial_states(
        &self,
    ) -> impl Iterator<Item = (Vector<2, i64>, Direction)> {
        let (width, height) = self.0.shape();
        let width = width as i64;
        let height = height as i64;

        std::iter::empty()
            .chain((0..width).map(|x| ([x, 0].into(), Direction::Down)))
            .chain(
                (0..width)
                    .map(move |x| ([x, height - 1].into(), Direction::Up)),
            )
            .chain((0..height).map(|y| ([0, y].into(), Direction::Right)))
            .chain(
                (0..height)
                    .map(move |y| ([width - 1, y].into(), Direction::Left)),
            )
    }
}

impl DirectedGraph<(Vector<2, i64>, Direction)> for BeamMap {
    fn connections_from<'a>(
        &'a self,
        (old_pos, old_dir): &'a (Vector<2, i64>, Direction),
    ) -> impl Iterator<Item = (Vector<2, i64>, Direction)> + '_ {
        self.0
            .get(*old_pos)
            .into_iter()
            .flat_map(|tile| {
                tile.outgoing_direction(*old_dir)
                    .map(move |dir| (tile, dir))
            })
            // .inspect(move |(tile, out_dir)| {
            //     let out_pos = *old_pos + out_dir.as_vec();
            //     println!(
            //         "When entering location {old_pos} \
            //          headed in direction {old_dir:?}, \
            //          will encounter {tile:?} \
            //          and then leave toward location {out_pos}, \
            //          headed in direction {out_dir:?}."
            //     )
            // })
            .map(|(_, out_dir)| out_dir)
            .map(move |out_dir| (*old_pos + out_dir.as_vec(), out_dir))
            .filter(|(out_pos, _)| self.0.is_valid(*out_pos))
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = BeamMap;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let map = lines.collect();
        Ok(BeamMap(map))
    }

    fn part_1(
        beam_map: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let initial_pos: Vector<2, i64> = [0, 0].into();
        let initial_dir = Direction::Right;
        let tiles_energized = beam_map
            .iter_depth_first([(initial_pos, initial_dir)])
            .map(|(loc, _)| loc)
            .unique()
            .count();

        // println!("{}", {
        //     let energized_tiles: std::collections::HashSet<_> = beam_map
        //         .iter_depth_first([(initial_pos, initial_dir)])
        //         .map(|(loc, _)| loc)
        //         .collect();
        //     beam_map.0.map(|(loc, tile)| {
        //         let style = if energized_tiles.contains(&loc) {
        //             console::Style::new().green().bright().bold()
        //         } else {
        //             console::Style::new()
        //         };
        //         style.apply_to(tile)
        //     })
        // });

        Ok(tiles_energized)
    }

    fn part_2(
        beam_map: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let max_tiles_energized = beam_map
            .initial_states()
            .map(|initial| {
                beam_map
                    .iter_depth_first([initial])
                    .map(|(loc, _)| loc)
                    .unique()
                    .count()
            })
            .max()
            .unwrap();
        Ok(max_tiles_energized)
    }
}
