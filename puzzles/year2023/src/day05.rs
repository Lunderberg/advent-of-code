use std::{ops::Range, str::FromStr};

use aoc_utils::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Category {
    Seed,
    Soil,
    Fertilizer,
    Water,
    Light,
    Temperature,
    Humidity,
    Location,
}

#[derive(Debug)]
pub struct Mapping {
    source: (Category, usize),
    dest: (Category, usize),
    extent: usize,
}

#[derive(Debug)]
pub struct Almanac {
    mappings: Vec<Mapping>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Item {
    category: Category,
    id: Range<usize>,
}

impl FromStr for Category {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "seed" | "Seed" => Ok(Category::Seed),
            "soil" | "Soil" => Ok(Category::Soil),
            "fertilizer" | "Fertilizer" => Ok(Category::Fertilizer),
            "water" | "Water" => Ok(Category::Water),
            "light" | "Light" => Ok(Category::Light),
            "temperature" | "Temperature" => Ok(Category::Temperature),
            "humidity" | "Humidity" => Ok(Category::Humidity),
            "location" | "Location" => Ok(Category::Location),
            _ => Err(Error::InvalidString(s.to_string())),
        }
    }
}

impl Mapping {
    fn source_range(&self) -> Range<usize> {
        let begin = self.source.1;
        begin..(begin + self.extent)
    }
}

impl Almanac {
    fn apply(&self, item: Item) -> impl Iterator<Item = Item> + '_ {
        let Item { category, id } = item.clone();
        let mut ranges = vec![id];

        std::iter::from_fn(move || {
            let range = ranges.pop()?;

            let mut out_category: Option<Category> = None;

            self.mappings
                .iter()
                .filter(|mapping| mapping.source.0 == category)
                .find_map(|mapping| {
                    out_category = Some(mapping.dest.0);
                    let src_range_covered =
                        mapping.source_range().intersection(&range)?;
                    let dest_range_start = src_range_covered.start
                        - mapping.source.1
                        + mapping.dest.1;
                    let dest_range_end = src_range_covered.end
                        - mapping.source.1
                        + mapping.dest.1;
                    let dest_range = dest_range_start..dest_range_end;
                    Some((src_range_covered, dest_range))
                })
                .map(|(src_range_covered, dest_range)| {
                    if src_range_covered.end != range.end {
                        ranges.push(src_range_covered.end..range.end);
                    }
                    if src_range_covered.start != range.start {
                        ranges.push(range.start..src_range_covered.start);
                    }

                    Item {
                        category: out_category.unwrap(),
                        id: dest_range,
                    }
                })
                .or_else(|| {
                    out_category.map(|out_cat| Item {
                        category: out_cat,
                        id: range,
                    })
                })
        })
    }
}

impl DynamicGraph<Item> for Almanac {
    fn connections_from(&self, node: &Item) -> Vec<(Item, u64)> {
        self.apply(node.clone())
            .map(|new_node| (new_node, 1))
            .collect()
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = (Vec<usize>, Almanac);
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let seed_line = lines.next().unwrap();
        let initial = seed_line
            .split_ascii_whitespace()
            .skip(1)
            .map(|seed_num| seed_num.parse())
            .collect::<Result<Vec<_>, _>>()?;

        lines.next();

        let mappings = lines
            .batching(|iter| {
                let (categories, _) = iter
                    .next()?
                    .split_ascii_whitespace()
                    .collect_tuple()
                    .unwrap();
                let (source, _, dest) =
                    categories.split('-').collect_tuple().unwrap();
                let source_category = source.parse().unwrap();
                let dest_category = dest.parse().unwrap();

                let mapping_iter = iter
                    .take_while(|line| !line.is_empty())
                    .map(move |line| {
                        let (dest_start, source_start, extent) = line
                            .split_ascii_whitespace()
                            .map(|num| num.parse().unwrap())
                            .collect_tuple()
                            .unwrap();
                        Mapping {
                            source: (source_category, source_start),
                            dest: (dest_category, dest_start),
                            extent,
                        }
                    })
                    .collect::<Vec<_>>()
                    .into_iter();
                Some(mapping_iter)
            })
            .flatten()
            .collect();

        Ok((initial, Almanac { mappings }))
    }

    fn part_1(
        (initial, almanac): &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let initial = initial.iter().map(|&id| Item {
            category: Category::Seed,
            id: id..(id + 1),
        });

        let loc = almanac
            .iter_depth_first(initial)
            .filter(|item| matches!(item.category, Category::Location))
            .map(|item| item.id.start)
            .min();

        Ok(loc)
    }

    fn part_2(
        (initial, almanac): &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let initial = initial.iter().tuples().map(|(&a, &b)| Item {
            category: Category::Seed,
            id: a..(a + b),
        });

        let loc = almanac
            .iter_depth_first(initial)
            .filter(|item| matches!(item.category, Category::Location))
            .map(|item| item.id.start)
            .min();

        Ok(loc)
    }
}
