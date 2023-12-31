#![allow(dead_code)]

use std::{collections::HashMap, fmt::Display, str::FromStr};

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

struct AffineLinearSpace<const N: usize, T> {
    offset: Vector<N, T>,
    basis_states: Vec<Vector<N, T>>,
}

#[derive(Clone)]
struct AugmentedMatrix<const ROWS: usize, const COLS: usize, T> {
    matrix: Matrix<ROWS, COLS, T>,
    augment: Vector<ROWS, T>,
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
                // println!(
                //     "Normalizing line {i} ({}), dividing by {gcd}",
                //     self.matrix[i]
                // );

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
        // println!("Before normalization:\n{}", self);
        for row in 0..ROWS {
            self.normalize_equation(row);
        }

        let mut row = 0;

        for column in 0..COLS {
            // println!("Before column {column}:\n{}", self);
            if let Some(nonzero_row_i) =
                (row..ROWS).find(|&j| !self.matrix[j][column].is_zero())
            {
                if row != nonzero_row_i {
                    // println!(
                    //     "Swapping lines {row} and {nonzero_row_i} ({} and {})",
                    //     self.matrix[row], self.matrix[nonzero_row_i]
                    // );
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

                        // println!("Updating row {j} based on row {row}");
                        // println!("\tScaling row {j} by {},", a / gcd);
                        // println!("\tthen scaling row {row} by {}", b / gcd);
                        // println!(
                        //     "\tsubtracting {} from {}",
                        //     self.matrix[row] * (b / gcd),
                        //     self.matrix[j] * (a / gcd)
                        // );
                        // println!(
                        //     "\tresults in {}",
                        //     self.matrix[j] * (a / gcd)
                        //         - self.matrix[row] * (b / gcd)
                        // );

                        self.matrix[j] = self.matrix[j] * (a / gcd)
                            - self.matrix[row] * (b / gcd);

                        self.augment[j] = self.augment[j] * (a / gcd)
                            - self.augment[row] * (b / gcd);
                        // println!(
                        //     "After canceliing out {row} from {j}:\n{}",
                        //     self
                        // );
                        self.normalize_equation(j);

                        // println!("After re-normalizing {j}:\n{}", self);
                    }
                }
                row += 1;
            }

            // println!("After column {column}:\n{}", self);
        }

        self
    }

    fn solve_system(&self) -> Option<AffineLinearSpace<ROWS, Fraction<T>>>
    where
        T: Copy,
        T: num::Integer,
        T: num::Signed,
        T: Display,
    {
        let echelon_form = self.clone().row_echelon_form();
        println!("Echelon form:\n{}", echelon_form);
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
        let offset: Vector<ROWS, Fraction<T>> = (0..ROWS)
            .filter_map(|row| {
                leading_terms[row].map(|col| {
                    let value = Fraction {
                        num: solution[row],
                        denom: echelon_form[(row, col)],
                    };
                    Vector::<ROWS, _>::one_hot(col) * value.normalize()
                })
            })
            .sum();

        // Columns that do not contain a leading non-zero term are
        // used to determine the basis vectors of the solution space.
        let basis_states: Vec<Vector<ROWS, Fraction<T>>> = (0..COLS)
            .filter(|col| !leading_terms.iter().contains(&Some(*col)))
            .map(|col| {
                (0..ROWS)
                    .filter_map(|row| {
                        leading_terms[row].map(|leading_col| (row, leading_col))
                    })
                    .filter(|(_, leading_col)| *leading_col != col)
                    .map(|(row, leading_col)| {
                        Vector::<ROWS, _>::one_hot(leading_col)
                            * Fraction {
                                num: -echelon_form[(row, leading_col)],
                                denom: echelon_form[(row, col)],
                            }
                    })
                    .fold(Vector::one_hot(col), |a, b| a + b)
            })
            .collect();

        println!("Offset = {offset}");
        println!("Basis states = [{}]", basis_states.iter().join(",\n\t"));

        // (0..COLS)
        //     .map(|col| {
        //         (0..ROWS)
        //             .map(|row| (row,echelon_form[(row,col)]))
        //             .filter(|(_,element)| !element.is_zero())
        //             .map(|(row,element)| {
        //                 let leading_col = leading_terms[row].unwrap();
        //                 if col==leading_col {
        //                     // The non-zero element is the leading
        //                     // term in its row.  It should be used to
        //                     // determine a point that lies within the
        //                     // solution space.
        //                     let offset = Vector::<COLS, _>::one_hot(col)
        //                     * Fraction {
        //                         num: solution[row],
        //                         denom: element,
        //                     };
        //                 } else {
        //                     // The column does not contain any leading
        //                     // terms.  It is instead used to
        //                 }
        //             })
        //     });

        // // Any column that does not contain a leading term provides a bas

        // (0..COLS).flat_map(|col| {
        //     (0..ROWS)
        //         .map(|row| (row, &echelon_form[(row, col)]))
        //         .filter(|(_, element)| !element.is_zero())
        //         .map(|(row, current_element)| {
        //             (0..col)
        //                 .map(|lrow| (lrow, &echelon_form[(lrow, col)]))
        //                 .find(|(_, left_element)| !left_element.is_zero())
        //                 .map(|(leading_row, leading_element)| {
        //                     // If there is a previous non-zero element in this row, then the
        //                 });
        //         })
        // });

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
            .all(|(delta, v)| delta * v.into() >= 0.into());
        let is_p2_future = (pos - p2)
            .into_iter()
            .zip(v2.into_iter())
            .all(|(delta, v)| delta * v.into() >= 0.into());

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

        // println!("For self={self}, other={other}");
        // println!("\tdelta pos = {dp}");
        // println!("\tdelta vel = {dv}");
        // println!("\tdp_scale = {dp_scale}");
        // println!("\tdv_scale = {dv_scale}");
        // println!("\tdp norm = {}", dp / dp_scale);
        // println!("\tdv norm = {}", -dv / dv_scale);
        if dp / dp_scale == -dv / dv_scale {
            Some(dp_scale / dv_scale)
        } else {
            None
        }
    }

    // fn at_time(&self, time: i128) -> Vector<3, i128> {
    //     self.position + self.velocity * time
    // }

    fn newtons_method_jacobian(
        hail: [&Hail; 3],
        prev: Vector<9, i128>,
    ) -> AugmentedMatrix<9, 9, i128> {
        // p_i + v_i*t_i == p_rock + v_rock*t_i
        // 0 == f_i == p_i + vi*ti - p_rock - v_rock*ti
        //
        //  f_i == p_i + vi*ti - p_rock - v_rock*ti
        //
        // (d/p_rock)f_i = -1
        // (d/v_rock)f_i = -ti
        // (d/ti)f_i = vi - v_rock
        //
        //  f_i.x == p_i.x + vi.x*ti - p_rock.x - v_rock.x*ti
        //  f_i.x == (p_i.x - p_rock.x) + (vi.x - v_rock.x)*ti

        let p_rock = Vector::new([prev[0], prev[1], prev[2]]);
        let v_rock = Vector::new([prev[3], prev[4], prev[5]]);
        let t = [prev[6], prev[7], prev[8]];

        let dp = hail.map(|stone| stone.position - p_rock);
        let dv = hail.map(|stone| stone.velocity - v_rock);

        // let function_value: Vector<9, i128> = hail
        //     .iter()
        //     .zip(t.iter().cloned())
        //     .map(|(hail, ti)| {
        //         (hail.position - p_rock) + (hail.velocity - v_rock) * ti
        //     })
        //     .flat_map(|p| p.into_iter())
        //     .collect::<Vec<_>>()
        //     .try_into()
        //     .unwrap();

        let function_value: Vector<9, i128> = [
            dp[0].x() + dv[0].x() * t[0],
            dp[0].y() + dv[0].y() * t[0],
            dp[0].z() + dv[0].z() * t[0],
            dp[1].x() + dv[0].x() * t[1],
            dp[1].y() + dv[0].y() * t[1],
            dp[1].z() + dv[0].z() * t[1],
            dp[2].x() + dv[0].x() * t[2],
            dp[2].y() + dv[0].y() * t[2],
            dp[2].z() + dv[0].z() * t[2],
        ]
        .into();

        let jacobian = Matrix::new([
            // f_0.x
            [-1, 0, 0, -t[0], 0, 0, dv[0].x(), 0, 0],
            // f_0.y
            [0, -1, 0, 0, -t[0], 0, 0, dv[0].y(), 0],
            // f_0.z
            [0, 0, -1, 0, 0, -t[0], 0, 0, dv[0].z()],
            // f_1.x
            [-1, 0, 0, -t[1], 0, 0, dv[1].x(), 0, 0],
            // f_1.y
            [0, -1, 0, 0, -t[1], 0, 0, dv[1].y(), 0],
            // f_1.z
            [0, 0, -1, 0, 0, -t[1], 0, 0, dv[1].z()],
            // f_2.x
            [-1, 0, 0, -t[2], 0, 0, dv[2].x(), 0, 0],
            // f_2.y
            [0, -1, 0, 0, -t[2], 0, 0, dv[2].y(), 0],
            // f_2.z
            [0, 0, -1, 0, 0, -t[2], 0, 0, dv[2].z()],
        ]);

        AugmentedMatrix {
            matrix: jacobian,
            augment: -function_value,
        }
    }

    fn find_rock_position(hail: [&Hail; 3]) -> Hail {
        println!("Finding rock to hit 3 hail particles at ");
        println!("\t{}", hail[0]);
        println!("\t{}", hail[1]);
        println!("\t{}", hail[2]);

        //let initial_guess: Vector<9, i128> = [0; 9].into();
        let initial_guess: Vector<9, i128> = [0, 0, 0, 1, 1, 1, 1, 2, 3].into();
        // let initial_guess: Vector<9, i128> = [
        //     24, 13, 10, // p_rock
        //     -3, 1, 2, // v_rock
        //     5, 3, 4, // t0/t1/t2
        // ]
        // .into();

        println!(
            "Starting at p_rock = {}, \
             v_rock = {}, \
             (t0,t1,t2) = {}",
            Vector::new([initial_guess[0], initial_guess[1], initial_guess[2]]),
            Vector::new([initial_guess[3], initial_guess[4], initial_guess[5]]),
            Vector::new([initial_guess[6], initial_guess[7], initial_guess[8]]),
        );

        let solution = std::iter::successors(Some(initial_guess), |prev| {
            println!(
                "Improving guess from \
                     p_rock = {}, \
                     v_rock = {}, \
                     (t0,t1,t2) = {}",
                Vector::new([prev[0], prev[1], prev[2]]),
                Vector::new([prev[3], prev[4], prev[5]]),
                Vector::new([prev[6], prev[7], prev[8]]),
            );
            let jacobian = Self::newtons_method_jacobian(hail, *prev);

            println!("Jacobian:\n{}", jacobian);

            let converged = jacobian.augment.iter().all(|value| *value == 0);
            if converged {
                println!("\tFound zero-point, converged");
                None
            } else {
                let solution =
                    jacobian.solve_system().expect("Could not solve Jacobian");
                let delta = solution.offset;
                println!(
                    "\tUpdating by delta,  \
                         p_rock = {}, \
                         v_rock = {}, \
                         (t0,t1,t2) = {}",
                    Vector::new([delta[0], delta[1], delta[2]]),
                    Vector::new([delta[3], delta[4], delta[5]]),
                    Vector::new([delta[6], delta[7], delta[8]]),
                );

                let delta = delta.map(|fraction| fraction.round_nearest());
                println!(
                    "\tUpdating by delta,  \
                         p_rock = {}, \
                         v_rock = {}, \
                         (t0,t1,t2) = {}",
                    Vector::new([delta[0], delta[1], delta[2]]),
                    Vector::new([delta[3], delta[4], delta[5]]),
                    Vector::new([delta[6], delta[7], delta[8]]),
                );

                let mut next = *prev + delta;

                println!(
                    "\tNext values,  \
                         p_rock = {}, \
                         v_rock = {}, \
                         (t0,t1,t2) = {}",
                    Vector::new([next[0], next[1], next[2]]),
                    Vector::new([next[3], next[4], next[5]]),
                    Vector::new([next[6], next[7], next[8]]),
                );

                for ti in 6..9 {
                    if next[ti] < 1 {
                        next[ti] = 1;
                    }
                }
                for ta in 6..9 {
                    if (6..9)
                        .filter(|&tb| ta != tb)
                        .any(|tb| next[ta] == next[tb])
                    {
                        next[ta] = (next[ta]..)
                            .find(|&t| {
                                (6..9)
                                    .filter(|&tb| ta != tb)
                                    .all(|tb| t != next[tb])
                            })
                            .unwrap();
                    }
                }

                println!(
                    "\tRestricting to valid values for t,  \
                         p_rock = {}, \
                         v_rock = {}, \
                         (t0,t1,t2) = {}",
                    Vector::new([next[0], next[1], next[2]]),
                    Vector::new([next[3], next[4], next[5]]),
                    Vector::new([next[6], next[7], next[8]]),
                );

                Some(next)
            }
        })
        .last()
        .unwrap();

        let p_rock = [solution[0], solution[1], solution[2]].into();
        let v_rock = [solution[3], solution[4], solution[5]].into();

        println!("v_rock = {v_rock}");
        println!("p_rock = {p_rock}");

        Hail {
            velocity: v_rock,
            position: p_rock,
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
        println!("Num hail: {}", storm.hail.len());

        let pos_lookup: HashMap<_, _> = storm
            .hail
            .iter()
            .enumerate()
            .flat_map(|(i, hail)| {
                (0..=storm.hail.len())
                    .map(move |t| {
                        let t = t as i128;
                        hail.position + hail.velocity * t
                    })
                    .map(move |pos| (pos, i))
            })
            .collect();
        println!("Lookup size: {}", pos_lookup.len());

        // pi = p0 + v0*ti

        // let best_rock = Hail {
        //     position: [24, 13, 10].into(),
        //     velocity: [-3, 1, 2].into(),
        // };

        let best_rock = Hail::find_rock_position(
            storm
                .hail
                .iter()
                .take(3)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );

        // let best_rock = storm
        //     .iter_pairs()
        //     .flat_map(|(a, b)| [(a.clone(), b.clone()), (b, a)])
        //     // .progress_count((storm.hail.len() * (storm.hail.len() - 1)) as u64)
        //     .map(|(a, b)| {
        //         let p1 = a.position + a.velocity * 1;
        //         let p2 = b.position + b.velocity * 2;
        //         let v0 = p2 - p1;
        //         let p0 = p1 - v0;
        //         // println!("From {a} at t=1 to {b} at t=2");
        //         // println!("\tp0 = {p0}, v0 = {v0}");
        //         Hail {
        //             position: p0,
        //             velocity: v0,
        //         }
        //     })
        //     .max_by_key(|rock| {
        //         (0..=storm.hail.len())
        //             .filter_map(|t| {
        //                 let t = t as i128;
        //                 let pos = rock.position + rock.velocity * t;
        //                 pos_lookup.get(&pos)
        //             })
        //             .unique()
        //             .count()
        //     })
        //     .unwrap();

        // let best_rock_collisions = (0..=storm.hail.len())
        //     .filter_map(|t| {
        //         let t = t as i128;
        //         let pos = best_rock.position + best_rock.velocity * t;
        //         pos_lookup.get(&pos)
        //     })
        //     .unique()
        //     .count();

        let best_rock_collisions = storm
            .hail
            .iter()
            .filter(|other| best_rock.intersection_time(other).is_some())
            .count();

        println!("Best rock: {best_rock:?}");
        println!("Num collisions: {best_rock_collisions}");

        Ok(best_rock.position.into_iter().sum::<i128>())
    }
}
