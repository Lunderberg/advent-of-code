use std::{collections::HashMap, fmt::Display, str::FromStr};

use aoc_utils::prelude::*;

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

trait MatrixExt {
    fn row_echelon_form(&self) -> Self;

    // TODO: Remove this function after `generic_const_exprs` can be
    // used.  Would be cleaner to implement in terms of an
    // `augment_column` function with signature below.
    //
    // fn augment_column(self, col: Vector<M,T>) -> Matrix<{N+1},M,T>;
    //
    // However, this would require const expressions for the return
    // type.
    type Column;
    fn row_echelon_form_with_augment(
        self,
        augment: Self::Column,
    ) -> (Self, Self::Column)
    where
        Self: Sized;

    type Rhs;
    type Solution;
    fn solve_system(&self, rhs: Self::Rhs) -> Self::Solution;
}
impl<const M: usize, const N: usize, T> MatrixExt for Matrix<M, N, T>
where
    T: Copy,
    T: num::Integer,
    T: num::Signed,
    T: Display,
{
    fn row_echelon_form(&self) -> Self {
        todo!()
    }

    type Column = Vector<M, T>;
    fn row_echelon_form_with_augment(
        mut self,
        mut augment: Vector<M, T>,
    ) -> (Self, Self::Column)
    where
        Self: Sized,
    {
        use num::integer::gcd as find_gcd;

        let normalize_line = |this: &mut Matrix<M, N, T>,
                              augment: &mut Vector<M, T>,
                              i: usize| {
            // Not technically required here, but canceling out
            // unnecessary factors in each row avoids some integer
            // overflow cases.
            if let Some(gcd) = this[i]
                .iter()
                .chain(std::iter::once(&augment[i]))
                .cloned()
                .reduce(find_gcd)
            {
                let leading_sign = this[i]
                    .iter()
                    .find(|val| !val.is_zero())
                    .map(|val| val.signum())
                    .unwrap_or(T::one());
                let gcd = gcd * leading_sign;

                if !gcd.is_zero() {
                    // println!(
                    //     "Normalizing line {i} ({}), dividing by {gcd}",
                    //     this[i]
                    // );

                    this[i] = this[i] / gcd;
                    augment[i] = augment[i] / gcd;
                }
            }
        };

        for i in 0..M {
            normalize_line(&mut self, &mut augment, i);
        }

        for i in 0..M {
            // println!(
            //     "Before line {i}: {}",
            //     self.display().line_prefix("               ")
            // );
            // println!("Before line {i}: {augment}");
            if let Some(nonzero_row_i) =
                (i..M).find(|&j| !T::is_zero(&self[j][i]))
            {
                if i != nonzero_row_i {
                    // println!(
                    //     "Swapping lines {i} and {nonzero_row_i} ({} and {})",
                    //     self[i], self[nonzero_row_i]
                    // );
                    self.swap_rows(i, nonzero_row_i);
                    augment.swap(i, nonzero_row_i);
                }

                // This loop could be reduced to only cover (i+1..M)
                // and still produce a correct row echelon form, since
                // that would provide the leading zeros for later
                // rows.  This way, the only step remaining for rref
                // is to scale each row to have a leading value of
                // one.  This isn't done by default, as it would also
                // require changing the type from T to Fraction<T>.
                for j in 0..M {
                    if i != j && !self[j][i].is_zero() {
                        let a = self[i][i];
                        let b = self[j][i];
                        let gcd = find_gcd(a, b);

                        // println!("Updating line {j} based on line {i}");
                        // println!("\tScaling line {j} by {},", a / gcd);
                        // println!("\tthen scaling line {i} by {}", b / gcd);
                        // println!(
                        //     "\tsubtracting {} from {}",
                        //     self[i] * (b / gcd),
                        //     self[j] * (a / gcd)
                        // );
                        // println!(
                        //     "\tresults in {}",
                        //     self[j] * (a / gcd) - self[i] * (b / gcd)
                        // );

                        self[j] = self[j] * (a / gcd) - self[i] * (b / gcd);

                        augment[j] =
                            augment[j] * (a / gcd) - augment[i] * (b / gcd);
                        normalize_line(&mut self, &mut augment, j);
                    }
                }
            }
            // println!(
            //     "After line {i}: {}",
            //     self.display().line_prefix("              ")
            // );
            // println!("After line {i}: {augment}");
        }

        (self, augment)
    }

    type Rhs = Vector<M, T>;

    type Solution = Option<Vector<N, Fraction<T>>>;

    /// Solve the system `self*solution == rhs`.
    ///
    /// Where `self` has shape `[M,N]`, the solution has shape
    /// `[N,1]`, and the right-hand side has shape `[M,1]`.
    fn solve_system(&self, rhs: Self::Rhs) -> Self::Solution {
        // println!("rhs = {rhs}");
        let (echelon_form, solution) = self.row_echelon_form_with_augment(rhs);

        // println!("[M,N] = [{M}, {N}]");
        // println!(
        //     "Echelon form = {}",
        //     echelon_form.display().line_prefix("               ")
        // );
        // println!("Solution: {solution}");

        let row_of_zeros: [bool; M] = std::array::from_fn(|i| {
            (0..N).all(|j| T::is_zero(&echelon_form[(i, j)]))
        });

        let rank = row_of_zeros.iter().map(|b| !b as usize).sum::<usize>();
        assert!(rank <= N);
        assert!(rank <= M);

        // The rank of the matrix may be insufficient to constrain the
        // system.  (Maybe I should bail out early if `M < N`?  The
        // rank cannot exceed `min(M,N)`, so that would skip the row
        // echelon form step.)
        let is_fully_constrained = rank == N;

        // If the system of equations is inconsistent, the LHS will
        // contain a fully-canceled row, but the RHS will not cancel
        // out entirely.
        let is_consistent = solution
            .iter()
            .enumerate()
            .filter(|(i, _)| row_of_zeros[*i])
            .all(|(_, t)| T::is_zero(t));

        if is_fully_constrained && is_consistent {
            // For a solution to exist, the echelon form must have a
            // diagonal matrix matrix as the top `[N,N]` square,
            let is_diagonal = echelon_form
                .iter_flat()
                .enumerate()
                .map(|(i, t)| (i.div_euclid(N), i.rem_euclid(M), t))
                .filter(|(i, j, _)| i != j)
                .all(|(_, _, t)| T::is_zero(t));
            assert!(is_diagonal);

            Some(
                std::array::from_fn(|i| {
                    Fraction {
                        num: solution[i],
                        denom: echelon_form[(i, i)],
                    }
                    .normalize()
                })
                .into(),
            )
        } else {
            None
        }
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

    fn at_time(&self, time: i128) -> Vector<3, i128> {
        self.position + self.velocity * time
    }

    fn newtons_method_jacobian(
        hail: [&Hail; 3],
        time: [i128; 3],
    ) -> (Matrix<3, 3, i128>, Vector<3, i128>) {
        // Starting with the equations requiring the rock to intersect
        // with three distinct pieces of hail:
        //
        // p0 + v0*t0 == p_rock + v_rock*t0
        // p1 + v1*t1 == p_rock + v_rock*t1
        // p2 + v2*t2 == p_rock + v_rock*t2
        //
        // The p_rock and v_rock are the easiest to cancel out,
        // leaving the equation:
        //
        // 0 == (
        //     + (v1-v2)*t1*t2
        //     + (v0-v1)*t0*t1
        //     + (v2-v0)*t0*t2
        //     + (p2-p1)*t0
        //     + (p1-p0)*t2
        //     + (p0-p2)*t1
        // )
        //
        // Since this must hold for all three x/y/z coordinates, this
        // is three equations with three unknowns, and could be solved
        // explicitly for t0/t1/t2.  However, solving cubics is a
        // pain.  Therefore, iteratively approaching the solution with
        // Newton's method instead.
        //
        // df/dt0 = (
        //     + (v0-v1)*t1
        //     + (v2-v0)*t2
        //     + (p2-p1)
        // )
        //
        // df/dt1 = (
        //     + (v1-v2)*t2
        //     + (v0-v1)*t0
        //     + (p0-p2)
        // )
        //
        // df/dt2 = (
        //     + (v1-v2)*t1
        //     + (v2-v0)*t0
        //     + (p1-p0)
        // )

        let dv0 = hail[0].velocity - hail[1].velocity;
        let dv1 = hail[1].velocity - hail[2].velocity;
        let dv2 = hail[2].velocity - hail[0].velocity;
        let dp0 = hail[0].position - hail[2].position;
        let dp1 = hail[1].position - hail[0].position;
        let dp2 = hail[2].position - hail[1].position;

        let function_value = dv1 * time[1] * time[2]
            + dv0 * time[0] * time[1]
            + dv2 * time[0] * time[2]
            + dp2 * time[0]
            + dp1 * time[2]
            + dp0 * time[1];

        let df_dt0 = dv0 * time[1] + dv2 * time[2] + dp2;
        let df_dt1 = dv1 * time[2] + dv0 * time[0] + dp0;
        let df_dt2 = dv1 * time[1] + dv2 * time[0] + dp1;

        let jacobian = Matrix::new([df_dt0, df_dt1, df_dt2]).transpose();

        (jacobian, function_value)
    }

    fn find_rock_position(hail: [&Hail; 3]) -> Hail {
        println!("Finding rock to hit 3 hail particles at ");
        println!("\t{}", hail[0]);
        println!("\t{}", hail[1]);
        println!("\t{}", hail[2]);

        let initial_guess: [i128; 3] = [0, 1, 2];
        // let initial_guess: [i128; 3] = [5, 3, 4];

        println!(
            "Starting at (t0,t1,t2) = ({}, {}, {})",
            initial_guess[0], initial_guess[1], initial_guess[2]
        );

        let collision_times =
            std::iter::successors(Some(initial_guess), |prev_time| {
                println!(
                    "Improving guess from (t0,t1,t2) = ({}, {}, {})",
                    prev_time[0], prev_time[1], prev_time[2]
                );
                let (jacobian, function_value) =
                    Self::newtons_method_jacobian(hail, *prev_time);

                println!("\tf(t0,t1,t2) = {function_value}");
                println!(
                    "\tJ(f) = {}",
                    jacobian.display().line_prefix("\t       ")
                );

                let converged = function_value.iter().all(|value| *value == 0);
                if converged {
                    println!("\tFound zero-point, converged");
                    None
                } else {
                    //let delta = jacobian * Vector::new([1; 3]);
                    let delta = jacobian
                        .solve_system(-function_value)
                        .expect("Could not solve Jacobian");
                    println!("\tUpdating by delta = {delta}");
                    let delta = delta.map(|fraction| fraction.round_nearest());
                    println!("\tUpdating by delta = {delta}");
                    let new_time = [
                        prev_time[0] + delta[0],
                        prev_time[1] + delta[1],
                        prev_time[2] + delta[2],
                    ];
                    println!(
                        "\tNext guess = ({}, {}, {})",
                        new_time[0], new_time[1], new_time[2]
                    );
                    Some(new_time)
                }
            })
            .last()
            .unwrap();

        let p_collision_0 = hail[0].at_time(collision_times[0]);
        let p_collision_1 = hail[1].at_time(collision_times[1]);
        let dt = collision_times[1] - collision_times[0];

        println!(
            "p_collision_0 = {p_collision_0} at t = {}",
            collision_times[0]
        );
        println!(
            "p_collision_1 = {p_collision_1} at t = {}",
            collision_times[1]
        );
        println!("dt = {dt}");

        let v_rock = (p_collision_1 - p_collision_0) / dt;
        let p_rock = p_collision_0 - v_rock * collision_times[0];

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
