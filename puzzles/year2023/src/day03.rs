use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use aoc_utils::{prelude::*, Adjacency};

use console::Style;

pub struct Schematic {
    tiles: GridMap<Tile>,
}

#[derive(PartialEq, Eq)]
enum Tile {
    None,
    Digit(u8),
    Symbol(char),
}

#[derive(Debug, Clone)]
struct Component {
    kind: ComponentKind,
    top_left: GridPos,
    bottom_right: GridPos,
}
#[derive(Debug, Clone)]
enum ComponentKind {
    Value(u32),
    Symbol(char),
}

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::None),
            '0'..='9' => Ok(Tile::Digit(c.to_digit(10).unwrap() as u8)),
            c if c.is_ascii_punctuation() => Ok(Tile::Symbol(c)),
            _ => Err(Error::UnknownChar(c)),
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tile::None => write!(f, "."),
            Tile::Digit(d) => write!(f, "{d}"),
            Tile::Symbol(c) => write!(f, "{c}"),
        }
    }
}

impl Display for Schematic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tiles)
    }
}

impl Schematic {
    fn iter_components(&self) -> impl Iterator<Item = Component> + '_ {
        self.tiles.iter().peekable().batching(|iter| {
            while iter
                .next_if(|(_, tile)| matches!(tile, Tile::None))
                .is_some()
            {}

            iter.next().map(|(top_left, tile)| {
                let (bottom_right, kind) = match tile {
                    Tile::None => unreachable!("Already filtered None"),
                    Tile::Symbol(c) => (top_left, ComponentKind::Symbol(*c)),
                    Tile::Digit(d) => {
                        let (bottom_right, value) = iter
                            .peeking_take_while(|(_, peek)| {
                                matches!(peek, Tile::Digit(_))
                            })
                            .fold((top_left, *d as u32), |(_, a), (loc, b)| {
                                let Tile::Digit(b) = b else { unreachable!("Alterady gfiltered to only Tile::Digit") };
                                let b = *b as u32;
                                (loc, 10 * a + b)
                            });
                        (bottom_right, ComponentKind::Value(value))
                    }
                };
                Component {
                    kind,
                    top_left,
                    bottom_right,
                }
            })
        })
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Schematic;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let tiles = lines
            .flat_map(|line| line.chars().chain(std::iter::once('\n')))
            .collect();
        Ok(Schematic { tiles })
    }

    fn part_1(
        schematic: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let adjacent_to_symbol: HashSet<GridPos> = schematic
            .iter_components()
            .filter_map(|comp| match comp.kind {
                ComponentKind::Symbol(_) => Some(comp.top_left),
                _ => None,
            })
            .flat_map(|loc| {
                schematic.tiles.adjacent_points(loc, Adjacency::Queen)
            })
            .collect();

        let is_adjacent_to_symbol = |comp: &Component| -> bool {
            schematic
                .tiles
                .iter_rect(comp.top_left, comp.bottom_right)
                .any(|(loc, _)| adjacent_to_symbol.contains(&loc))
        };

        let value = schematic
            .iter_components()
            .filter(|comp| match comp.kind {
                ComponentKind::Value(_) => is_adjacent_to_symbol(comp),
                _ => false,
            })
            .filter_map(|comp| match comp.kind {
                ComponentKind::Value(value) => Some(value),
                _ => None,
            })
            .sum::<u32>();

        println!("{}", {
            let style_green = Style::new().green().bright().bold();
            let style_red = Style::new().red();
            let styled: HashMap<_, _> = schematic
                .iter_components()
                .filter(|comp| matches!(comp.kind, ComponentKind::Value(_)))
                .flat_map(|comp| {
                    let style = if is_adjacent_to_symbol(&comp) {
                        &style_green
                    } else {
                        &style_red
                    };
                    schematic
                        .tiles
                        .iter_rect(comp.top_left, comp.bottom_right)
                        .map(move |(loc, _)| {
                            (loc.as_vec(&schematic.tiles), style)
                        })
                })
                .collect();

            schematic.tiles.map(|loc, tile| {
                styled.get(&loc).map_or_else(
                    || Style::new().apply_to(tile),
                    |style| style.apply_to(tile),
                )
            })
        });

        Ok(value)
    }

    fn part_2(
        schematic: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let numbers: Vec<_> = schematic
            .iter_components()
            .filter(|comp| matches!(comp.kind, ComponentKind::Value(_)))
            .collect();

        let as_gear = |comp: &Component| -> Option<(&Component, &Component)> {
            match comp.kind {
                ComponentKind::Symbol('*') => numbers
                    .iter()
                    .filter(|num| {
                        schematic
                            .tiles
                            .iter_rect(num.top_left, num.bottom_right)
                            .any(|(pos, _)| {
                                schematic
                                    .tiles
                                    .adjacent_points(
                                        comp.top_left,
                                        Adjacency::Queen,
                                    )
                                    .any(|adj| pos == adj)
                            })
                    })
                    .collect_tuple(),
                _ => None,
            }
        };

        let value = schematic
            .iter_components()
            .filter_map(|comp| as_gear(&comp))
            .map(|(a, b)| {
                let ComponentKind::Value(a) = a.kind else {
                    panic!()
                };
                let ComponentKind::Value(b) = b.kind else {
                    panic!()
                };
                a * b
            })
            .sum::<u32>();

        println!("{}", {
            let style_green = Style::new().green().bright().bold();
            let style_red = Style::new().red();

            let styled: HashMap<_, _> = schematic
                .iter_components()
                .flat_map(|comp| {
                    matches!(comp.kind, ComponentKind::Symbol('*'))
                        .then(|| {
                            let comp_as_gear = as_gear(&comp);
                            std::iter::empty::<(Component, bool)>()
                            .chain(std::iter::once((
                                comp.clone(),
                                comp_as_gear.is_some(),
                            )))
                            .chain(
                                comp_as_gear
                                    .into_iter()
                                    .flat_map(
                                        |(a, b): (&Component, &Component)| {
                                            [a, b].into_iter()
                                        },
                                    )
                                    .map(|comp: &Component| {
                                        (comp.clone(), true)
                                    }),
                            )
                        })
                        .into_iter()
                        .flatten()
                })
                .flat_map(|(comp, is_gear)| {
                    let style = if is_gear { &style_green } else { &style_red };
                    schematic
                        .tiles
                        .iter_rect(comp.top_left, comp.bottom_right)
                        .map(move |(loc, _)| {
                            (loc.as_vec(&schematic.tiles), style)
                        })
                })
                .collect();

            schematic.tiles.map(|loc, tile| {
                styled.get(&loc).map_or_else(
                    || Style::new().apply_to(tile),
                    |style| style.apply_to(tile),
                )
            })
        });

        Ok(value)
    }
}
