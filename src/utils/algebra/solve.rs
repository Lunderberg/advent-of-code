use super::{Expression, Variable};

use std::collections::{HashMap, HashSet, VecDeque};

use itertools::Itertools;

pub struct SystemSolution {
    pub definitions: HashMap<Variable, Expression>,
    pub implied_constraints: Vec<Expression>,
}

impl Expression {
    pub fn solve_for(&self, var: Variable) -> Option<Expression> {
        Self::solve_for_impl(var, self.clone(), 1.into())
            .map(|expr| expr.simplify())
    }

    fn solve_for_impl(
        var: Variable,
        left: Expression,
        right: Expression,
    ) -> Option<Expression> {
        let (var_expr, const_expr) =
            match (left.has_variable(var), right.has_variable(var)) {
                (true, true) => None,
                (true, false) => Some((left, right)),
                (false, true) => Some((right, left)),
                (false, false) => None,
            }?;

        use Expression::*;
        match (var_expr, const_expr) {
            (Impossible, _) | (_, Impossible) => None,
            (Variable(_), expr) => Some(expr),
            (Equal(boxed), Int(1)) => {
                let (a, b) = *boxed;
                Self::solve_for_impl(var, a, b)
            }
            (Not(boxed), expr) => {
                Self::solve_for_impl(var, *boxed, Not(Box::new(expr)))
            }
            _ => None,
        }
    }

    pub fn solve_system(
        constraints: &Vec<Expression>,
        known_vars: &Vec<Variable>,
    ) -> SystemSolution {
        let known_vars: HashSet<Variable> =
            known_vars.iter().copied().collect();
        let mut definitions: HashMap<Variable, Expression> = known_vars
            .iter()
            .copied()
            .map(|var| (var, var.into()))
            .collect();
        let implied_constraints: Vec<Expression> = Vec::new();

        let mut to_check: VecDeque<Expression> =
            constraints.iter().cloned().collect();
        let mut checked_since_last_success = 0;

        while to_check.len() > 0 && checked_since_last_success < to_check.len()
        {
            let equality = to_check.pop_front().unwrap();

            let solved_var = equality
                .variables()
                .into_iter()
                .filter(|var| !definitions.contains_key(var))
                .sorted()
                .rev()
                .find_map(|var| {
                    equality.solve_for(var).map(move |expr| (var, expr))
                });

            if let Some((var, expr)) = solved_var {
                println!("Found {} is {}", var, expr);
                // Update the existing equalities and derived
                // expressions.
                to_check
                    .iter_mut()
                    .filter(|prev_expr| prev_expr.has_variable(var))
                    .for_each(|prev_expr| {
                        *prev_expr = prev_expr.substitute(var, &expr).simplify()
                    });
                definitions
                    .iter_mut()
                    .map(|(_prev_var, prev_expr)| prev_expr)
                    .filter(|prev_expr| prev_expr.has_variable(var))
                    .for_each(|prev_expr| {
                        *prev_expr = prev_expr.substitute(var, &expr).simplify()
                    });

                // Mark this variable as known
                definitions.insert(var, expr);

                checked_since_last_success = 0;
            } else {
                // Push the equality back onto the queue, maybe it'll
                // be easier to solve next time around.
                to_check.push_back(equality);
                checked_since_last_success += 1;
            }
        }

        SystemSolution {
            definitions,
            implied_constraints,
        }
    }
}
