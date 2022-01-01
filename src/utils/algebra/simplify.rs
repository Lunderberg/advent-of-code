use super::{Expression, Variable};

impl Expression {
    pub fn simplify(self) -> Self {
        let mut constraints = Vec::new();
        self.simplify_impl(&mut constraints)
    }

    fn simplify_impl(
        self,
        constraints: &mut Vec<(Variable, Expression)>,
    ) -> Self {
        use Expression::*;

        match self {
            Impossible | Variable(_) | Int(_) => self,

            Add(boxed) => {
                let a = boxed.0.simplify_impl(constraints);
                let b = boxed.1.simplify_impl(constraints);
                match (a, b) {
                    (Int(a), Int(b)) => Int(a + b),
                    (Int(0), b) => b,
                    (a, Int(0)) => a,
                    (a, b) => Add(Box::new((a, b))),
                }
            }
            Sub(boxed) => {
                let a = boxed.0.simplify_impl(constraints);
                let b = boxed.1.simplify_impl(constraints);
                match (a, b) {
                    (Int(a), Int(b)) => Int(a - b),
                    (Int(0), b) => Mul(Box::new(((-1).into(), b))),
                    (a, Int(0)) => a,
                    (a, b) => Sub(Box::new((a, b))),
                }
            }
            Mul(boxed) => {
                let a = boxed.0.simplify_impl(constraints);
                let b = boxed.1.simplify_impl(constraints);
                match (a, b) {
                    (Int(a), Int(b)) => Int(a * b),
                    (Int(1), b) => b,
                    (a, Int(1)) => a,
                    (Int(0), _) => Int(0),
                    (_, Int(0)) => Int(0),
                    (a, b) => Mul(Box::new((a, b))),
                }
            }
            Div(boxed) => {
                let a = boxed.0.simplify_impl(constraints);
                let b = boxed.1.simplify_impl(constraints);
                match (a, b) {
                    (Int(a), Int(b)) => Int(a / b),
                    (a, Int(1)) => a,
                    (Int(0), _) => Int(0),
                    (_, Int(0)) => Impossible,
                    (a, b) => Div(Box::new((a, b))),
                }
            }
            Mod(boxed) => {
                let a = boxed.0.simplify_impl(constraints);
                let b = boxed.1.simplify_impl(constraints);
                match (a, b) {
                    (Int(a), Int(b)) => Int(a % b),
                    (_, Int(0)) => Impossible,
                    (_a, Int(1)) => Int(0),
                    (Int(0), _) => Int(0),
                    (Int(1), _) => Int(1),
                    (a, b) => Mod(Box::new((a, b))),
                }
            }
            Equal(boxed) => {
                let a = boxed.0.simplify_impl(constraints);
                let b = boxed.1.simplify_impl(constraints);
                match (a, b) {
                    (Int(a), Int(b)) => Int((a == b) as i64),
                    (a, Int(0)) => Not(Box::new(a)),
                    (Int(0), b) => Not(Box::new(b)),
                    (a, b) => Equal(Box::new((a, b))),
                }
            }
            Not(boxed) => {
                let a = boxed.simplify_impl(constraints);
                match a {
                    Int(a) => Int((a == 0) as i64),
                    a => Not(Box::new(a)),
                }
            }
            IfThenElse(boxed) => {
                let a = boxed.0.simplify_impl(constraints);
                let b = boxed.1.simplify_impl(constraints);
                let c = boxed.2.simplify_impl(constraints);
                match (a, b, c) {
                    (Int(1), b, _) => b,
                    (Int(0), _, c) => c,
                    (a, b, c) => IfThenElse(Box::new((a, b, c))),
                }
            }
        }
    }
}
