use std::{fmt::Display, str::FromStr};

use aoc_utils::prelude::*;
use num::integer::gcd as find_gcd;

pub struct Storm {
    hail: Vec<Hail>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Hail {
    position: Vector<3, i128>,
    velocity: Vector<3, i128>,
}

impl FromStr for Hail {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let (position, velocity) = line
            .split('@')
            .flat_map(|s| s.split(','))
            .map(|s| s.trim().parse())
            .tuples()
            .map(|(x, y, z)| -> Result<_, Error> { Ok([x?, y?, z?].into()) })
            .collect_tuple()
            .ok_or(Error::WrongIteratorSize)?;

        Ok(Hail {
            position: position?,
            velocity: velocity?,
        })
    }
}
impl Display for Hail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{pos {}, vel {}}}", self.position, self.velocity)
    }
}
impl Display for Storm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.hail.iter().try_for_each(|hail| writeln!(f, "{hail}"))
    }
}

trait VectorExt {
    type T;
    fn norm1(&self) -> Self::T;
}
impl<const N: usize, T> VectorExt for Vector<N, T>
where
    T: Ord,
    T: Copy,
    T: num::Zero,
    T: std::ops::Sub<Output = T>,
{
    type T = T;
    fn norm1(&self) -> Self::T {
        assert!(N > 0);
        self.iter()
            .map(|dim| {
                if T::zero() <= *dim {
                    *dim
                } else {
                    T::zero() - *dim
                }
            })
            .max()
            .unwrap()
            .clone()
    }
}

pub struct AffineLinearSpace<const N: usize, T> {
    pub offset: Vector<N, T>,
    pub basis_states: Vec<Vector<N, T>>,
}

#[derive(Clone)]
pub struct AugmentedMatrix<const ROWS: usize, const COLS: usize, T> {
    pub matrix: Matrix<ROWS, COLS, T>,
    pub augment: Vector<ROWS, T>,
}

impl<const ROWS: usize, const COLS: usize, T> std::fmt::Display
    for AugmentedMatrix<ROWS, COLS, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let col_widths: [usize; COLS] = std::array::from_fn(|i| {
            (0..ROWS)
                .map(|j| format!("{}", self.matrix[(j, i)]).len())
                .max()
                .unwrap_or(0)
        });
        let aug_width = (0..ROWS)
            .map(|j| format!("{}", self.augment[j]).len())
            .max()
            .unwrap_or(0);
        let total_width =
            col_widths.iter().map(|w| w + 2).sum::<usize>() + aug_width + 3;

        writeln!(f, "┌{:width$}┐", "", width = total_width)?;
        (0..ROWS).try_for_each(|j| {
            write!(f, "|")?;
            self.matrix[j]
                .iter()
                .zip(col_widths.iter())
                .try_for_each(|(item, width)| write!(f, " {item:width$} "))?;
            writeln!(f, "| {:aug_width$} |", self.augment[j])
        })?;
        writeln!(f, "└{:width$}┘", "", width = total_width)?;
        Ok(())
    }
}

impl<const ROWS: usize, const COLS: usize, T> AugmentedMatrix<ROWS, COLS, T> {
    fn normalize_equation(&mut self, i: usize)
    where
        T: Copy,
        T: num::Integer,
        T: num::Signed,
        T: Display,
    {
        // Not technically required here, but canceling out
        // unnecessary factors in each row avoids some integer
        // overflow cases.
        if let Some(gcd) = self.matrix[i]
            .iter()
            .chain(std::iter::once(&self.augment[i]))
            .cloned()
            .reduce(find_gcd)
        {
            let leading_sign = self.matrix[i]
                .iter()
                .find(|val| !val.is_zero())
                .map(|val| val.signum())
                .unwrap_or(T::one());
            let gcd = gcd * leading_sign;

            if !gcd.is_zero() {
                self.matrix[i] = self.matrix[i] / gcd;
                self.augment[i] = self.augment[i] / gcd;
            }
        }
    }

    fn row_echelon_form(mut self) -> Self
    where
        T: Copy,
        T: num::Integer,
        T: num::Signed,
        T: Display,
    {
        for row in 0..ROWS {
            self.normalize_equation(row);
        }

        let mut row = 0;

        for column in 0..COLS {
            if let Some(nonzero_row_i) =
                (row..ROWS).find(|&j| !self.matrix[j][column].is_zero())
            {
                if row != nonzero_row_i {
                    self.matrix.swap_rows(row, nonzero_row_i);
                    self.augment.swap(row, nonzero_row_i);
                }

                // This loop could be reduced to only cover (i+1..M)
                // and still produce a correct row echelon form, since
                // that would provide the leading zeros for later
                // rows.  This way, the only step remaining for rref
                // is to scale each row to have a leading value of
                // one.  This isn't done by default, as it would also
                // require changing the type from T to Fraction<T>.
                for j in 0..ROWS {
                    if row != j && !self.matrix[j][column].is_zero() {
                        let a = self.matrix[row][column];
                        let b = self.matrix[j][column];
                        let gcd = find_gcd(a, b);

                        self.matrix[j] = self.matrix[j] * (a / gcd)
                            - self.matrix[row] * (b / gcd);

                        self.augment[j] = self.augment[j] * (a / gcd)
                            - self.augment[row] * (b / gcd);
                        self.normalize_equation(j);
                    }
                }
                row += 1;
            }
        }

        self
    }

    fn solve_system(&self) -> Option<AffineLinearSpace<COLS, Fraction<T>>>
    where
        T: Copy,
        T: num::Integer,
        T: num::Signed,
        T: Display,
    {
        let echelon_form = self.clone().row_echelon_form();
        let AugmentedMatrix {
            matrix: echelon_form,
            augment: solution,
        } = echelon_form;

        let row_of_zeros: [bool; ROWS] = std::array::from_fn(|i| {
            (0..COLS).all(|j| echelon_form[(i, j)].is_zero())
        });

        let rank = row_of_zeros.iter().map(|b| !b as usize).sum::<usize>();
        assert!(rank <= ROWS);
        assert!(rank <= COLS);

        // If the system of equations is inconsistent, the LHS will
        // contain a fully-canceled row, but the RHS will not cancel
        // out entirely.
        let is_consistent = solution
            .iter()
            .enumerate()
            .filter(|(i, _)| row_of_zeros[*i])
            .all(|(_, t)| T::is_zero(t));

        if !is_consistent {
            return None;
        }

        // The rank of the matrix may be insufficient to constrain the
        // system.  (Maybe I should bail out early if `M < N`?  The
        // rank cannot exceed `min(M,N)`, so that would skip the row
        // echelon form step.)
        let _is_fully_constrained = rank == COLS;

        let leading_terms: [Option<usize>; ROWS] = std::array::from_fn(|row| {
            echelon_form[row]
                .iter()
                .enumerate()
                .find(|(_, element)| !element.is_zero())
                .map(|(col, _)| col)
        });

        // The leading non-zero term in each row is used to determine
        // a point that lies within the solution space.
        let offset: Vector<COLS, Fraction<T>> = (0..ROWS)
            .filter_map(|row| {
                leading_terms[row].map(|col| {
                    let value = Fraction {
                        num: solution[row],
                        denom: echelon_form[(row, col)],
                    };
                    Vector::<COLS, _>::one_hot(col) * value.normalize()
                })
            })
            .sum();

        // Columns that do not contain a leading non-zero term are
        // used to determine the basis vectors of the solution space.
        let basis_states: Vec<Vector<COLS, Fraction<T>>> = (0..COLS)
            .filter(|col| !leading_terms.iter().contains(&Some(*col)))
            .map(|col| {
                (0..ROWS)
                    .filter_map(|row| {
                        leading_terms[row].map(|leading_col| (row, leading_col))
                    })
                    .filter(|(_, leading_col)| *leading_col != col)
                    .map(|(row, leading_col)| {
                        Vector::<COLS, _>::one_hot(leading_col)
                            * Fraction {
                                num: -echelon_form[(row, leading_col)],
                                denom: echelon_form[(row, col)],
                            }
                    })
                    .fold(Vector::one_hot(col), |a, b| a + b)
            })
            .collect();

        Some(AffineLinearSpace {
            offset,
            basis_states,
        })
    }
}

impl Storm {
    fn iter_pairs(&self) -> impl Iterator<Item = (Hail, Hail)> + '_ {
        self.hail.iter().cloned().tuple_combinations()
    }
}

impl Hail {
    fn xy_intersection(
        &self,
        other: &Hail,
    ) -> Option<Vector<2, Fraction<i128>>> {
        let v1: Vector<2, Fraction<i128>> =
            [self.velocity.x().into(), self.velocity.y().into()].into();
        let p1: Vector<2, Fraction<i128>> =
            [self.position.x().into(), self.position.y().into()].into();

        let v2: Vector<2, Fraction<i128>> =
            [other.velocity.x().into(), other.velocity.y().into()].into();
        let p2: Vector<2, Fraction<i128>> =
            [other.position.x().into(), other.position.y().into()].into();

        // For first hailstone,
        //
        // x = v1.x*t + p1.x
        // y = v1.y*t + p1.y
        //
        // v1.y*x = v1.y*v1.x*t + v1.y*p1.x
        // v1.x*y = v1.x*v1.y*t + v1.x*p1.y
        //
        // v1.y*x - v1.x*y = v1.y*p1.x - v1.x*p1.y

        // Analogously, for second hailstone,
        // v2.y*x - v2.x*y = v2.y*p2.x - v2.x*p2.y

        // ┌              ┐┌   ┐   ┌                       ┐
        // | v1.y   -v1.x || x | = | v1.y*p1.x - v1.x*p1.y |
        // | v2.y   -v2.x || y |   | v2.y*p2.x - v2.x*p2.y |
        // └              ┘└   ┘   └                       ┘
        //
        // ┌              ┐-1    ┌             ┐
        // | v1.y   -v1.x |    = | -v2.x  v1.x | / (v2.y*v1.x - v2.x*v1.y)
        // | v2.y   -v2.x |      | -v2.y  v1.y |
        // └              ┘      └             ┘
        // D = (v2.y*v1.x - v2.x*v1.y)
        //
        // ┌   ┐    ┌              ┐┌                       ┐
        // | x | =  | -v2.x  v1.x  || v1.y*p1.x - v1.x*p1.y | / D
        // | y |    | -v2.y  v1.y  || v2.y*p2.x - v2.x*p2.y |
        // └   ┘    └              ┘└                       ┘
        // ┌   ┐    ┌                                                              ┐
        // | x | =  | -v2.x*(v1.y*p1.x - v1.x*p1.y) + v1.x*(v2.y*p2.x - v2.x*p2.y) | / D
        // | y |    | -v2.y*(v1.y*p1.x - v1.x*p1.y) + v1.y*(v2.y*p2.x - v2.x*p2.y) |
        // └   ┘    └                                                              ┘

        let d = (v2.y() * v1.x() - v2.x() * v1.y()).normalize();

        if d == 0 {
            return None;
        }

        let a = (v1.y() * p1.x() - v1.x() * p1.y()).normalize();
        let b = (v2.y() * p2.x() - v2.x() * p2.y()).normalize();

        let x = (v1.x() * b - v2.x() * a) / d;
        let y = (v1.y() * b - v2.y() * a) / d;
        let pos: Vector<2, Fraction<i128>> = [x, y].into();

        let is_p1_future = (pos - p1)
            .into_iter()
            .zip(v1.into_iter())
            .all(|(delta, v)| delta * v >= 0.into());
        let is_p2_future = (pos - p2)
            .into_iter()
            .zip(v2.into_iter())
            .all(|(delta, v)| delta * v >= 0.into());

        if is_p1_future && is_p2_future {
            Some(pos)
        } else {
            None
        }
    }

    fn intersection_time(&self, other: &Hail) -> Option<Fraction<i128>> {
        let dp: Vector<3, Fraction<i128>> =
            (other.position - self.position).map(|dim| dim.into());
        let dv: Vector<3, Fraction<i128>> =
            (other.velocity - self.velocity).map(|dim| dim.into());

        let dp_scale: Fraction<i128> = dp.norm1().into();
        let dv_scale: Fraction<i128> = dv.norm1().into();

        if dp / dp_scale == -dv / dv_scale {
            Some(dp_scale / dv_scale)
        } else {
            None
        }
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Storm;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let hail = lines.map(|line| line.parse()).collect::<Result<_, _>>()?;
        Ok(Storm { hail })
    }

    fn part_1(
        storm: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let test_area: std::ops::RangeInclusive<Fraction<i128>> =
            if storm.hail.len() == 5 {
                7.into()..=27.into()
            } else {
                200000000000000.into()..=400000000000000.into()
            };

        let num_intersect = storm
            .iter_pairs()
            .filter_map(|(a, b)| a.xy_intersection(&b))
            .filter(|intersection| {
                test_area.contains(&intersection.x())
                    && test_area.contains(&intersection.y())
            })
            .count();
        Ok(num_intersect)
    }

    fn part_2(
        storm: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        // p0 + v0*t0 == p_rock + v_rock*t0
        // p1 + v1*t1 == p_rock + v_rock*t1

        // (p1-p0) + v1*t1 - v0*t0 == v_rock*(t1-t0)

        // if v0==v1
        // (p1-p0) + v*(t1-t0) == v_rock*(t1-t0)

        // (p1-p0) == (v_rock-v)*(t1-t0)

        // (p1-p0)%(v_rock-v) == 0

        // The example doesn't have enough hailstones to fully
        // constrain the velocity with this method.
        let v_rock: Vector<3, i128> = if storm.hail.len() == 5 {
            Vector::new([-3, 1, 2])
        } else {
            std::array::from_fn(|dim| {
                let constraints: Vec<_> = storm
                    .hail
                    .iter()
                    .map(|hailstone| {
                        (hailstone.velocity[dim], hailstone.position[dim])
                    })
                    .sorted_by_key(|(_, p)| *p)
                    .into_group_map()
                    .into_iter()
                    .filter(|(_, positions)| positions.len() > 1)
                    .sorted_by_key(|(v, _)| *v)
                    .filter(|(_, p)| p.len() > 1)
                    .flat_map(|(velocity, positions)| {
                        let first = positions[0];
                        positions
                            .into_iter()
                            .skip(1)
                            .map(move |position| (position - first, velocity))
                    })
                    .collect();

                (1..)
                    .flat_map(|i| [i, -i])
                    .filter(|v_rock| {
                        constraints.iter().all(|(dp, velocity)| {
                            v_rock != velocity && dp % (v_rock - velocity) == 0
                        })
                    })
                    .next()
                    .unwrap()
            })
            .into()
        };

        let system: AugmentedMatrix<6, 5, i128> = {
            let p0 = storm.hail[0].position;
            let p1 = storm.hail[1].position;
            let dv0 = v_rock - storm.hail[0].velocity;
            let dv1 = v_rock - storm.hail[1].velocity;

            AugmentedMatrix {
                matrix: Matrix::new([
                    [1, 0, 0, dv0.x(), 0],
                    [0, 1, 0, dv0.y(), 0],
                    [0, 0, 1, dv0.z(), 0],
                    [1, 0, 0, 0, dv1.x()],
                    [0, 1, 0, 0, dv1.y()],
                    [0, 0, 1, 0, dv1.z()],
                ]),
                augment: [p0.x(), p0.y(), p0.z(), p1.x(), p1.y(), p1.z()]
                    .into(),
            }
        };

        let solution = system.solve_system();
        let p_rock: Vector<3, i128> = solution
            .expect("No solution for position")
            .offset
            .into_iter()
            .take(3)
            .map(|f| {
                assert_eq!(f.num % f.denom, 0);
                f.num / f.denom
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("Iterator should have 3 items");

        let best_rock = Hail {
            position: p_rock,
            velocity: v_rock,
        };

        let best_rock_collisions = storm
            .hail
            .iter()
            .filter(|other| best_rock.intersection_time(other).is_some())
            .count();

        assert_eq!(best_rock_collisions, storm.hail.len());

        Ok(best_rock.position.into_iter().sum::<i128>())
    }
}
