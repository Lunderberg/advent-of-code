use std::collections::HashSet;
use std::ops;
use std::sync::atomic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Variable {
    pub(crate) id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    // No expression exists that satisfies the constraints.  For
    // example, backpropagating [0,*,*,*] through the instruction `eql
    // x x` would result in [Impossible, *,*,*], because no value of x
    // can produce 0.
    Impossible,

    // An unconstrained variable, introduced in back propagation.
    Variable(Variable),

    // An integer literal
    Int(i64),

    // Unary NOT.  Not(0) = 1.  Not(x) = 0 for all other x.
    Not(Box<Expression>),

    // Binary operators
    Add(Box<(Expression, Expression)>),
    Sub(Box<(Expression, Expression)>),
    Mul(Box<(Expression, Expression)>),
    Div(Box<(Expression, Expression)>),
    Mod(Box<(Expression, Expression)>),
    Equal(Box<(Expression, Expression)>),

    // Ternary operator.  Any non-zero expression is treated as true.
    IfThenElse(Box<(Expression, Expression, Expression)>),
}

static ID_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

impl Variable {
    pub fn new() -> Self {
        let id = ID_COUNTER.fetch_add(1, atomic::Ordering::SeqCst);
        Self { id }
    }
}

impl Expression {
    pub fn equal_value(self, other: Self) -> Self {
        Expression::Equal(Box::new((self, other)))
    }

    pub fn has_variable(&self, check_for: Variable) -> bool {
        let check = |node: &Expression| node.has_variable(check_for);

        use Expression::*;
        match self {
            Variable(var) => *var == check_for,
            Not(boxed) => check(&**boxed),

            Add(boxed) | Sub(boxed) | Mul(boxed) | Div(boxed) | Mod(boxed)
            | Equal(boxed) => {
                let (a, b) = &**boxed;
                check(a) || check(b)
            }

            IfThenElse(boxed) => {
                let (a, b, c) = &**boxed;
                check(a) || check(b) || check(c)
            }
            _ => false,
        }
    }

    pub fn variables(&self) -> HashSet<Variable> {
        self.preorder_iter()
            .filter_map(|expr| {
                if let Expression::Variable(var) = expr {
                    Some(*var)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn substitute(
        &self,
        to_replace: Variable,
        replace_with: &Expression,
    ) -> Expression {
        let update =
            |node: &Expression| node.substitute(to_replace, replace_with);

        use Expression::*;
        match self {
            Impossible => Impossible,

            Variable(var) => {
                if *var == to_replace {
                    replace_with.clone()
                } else {
                    Variable(*var)
                }
            }

            Int(num) => Int(*num),

            Not(boxed) => Not(Box::new(update(boxed))),

            Add(boxed) => Add(Box::new((update(&boxed.0), update(&boxed.1)))),
            Sub(boxed) => Sub(Box::new((update(&boxed.0), update(&boxed.1)))),
            Mul(boxed) => Mul(Box::new((update(&boxed.0), update(&boxed.1)))),
            Div(boxed) => Div(Box::new((update(&boxed.0), update(&boxed.1)))),
            Mod(boxed) => Mod(Box::new((update(&boxed.0), update(&boxed.1)))),
            Equal(boxed) => {
                Equal(Box::new((update(&boxed.0), update(&boxed.1))))
            }
            IfThenElse(boxed) => IfThenElse(Box::new((
                update(&boxed.0),
                update(&boxed.1),
                update(&boxed.2),
            ))),
        }
    }
}

impl From<Variable> for Expression {
    fn from(var: Variable) -> Self {
        Self::Variable(var)
    }
}

impl From<i64> for Expression {
    fn from(num: i64) -> Self {
        Self::Int(num)
    }
}

impl From<&i64> for Expression {
    fn from(num: &i64) -> Self {
        Self::Int(*num)
    }
}

impl ops::Not for Expression {
    type Output = Expression;
    fn not(self) -> Self {
        use Expression::*;
        match self {
            Impossible => Impossible,
            Int(a) => match a {
                0 => 1i64.into(),
                1 => 0i64.into(),
                _ => Impossible,
            },
            Not(a) => *a,
            _ => Not(Box::new(self)),
        }
    }
}

macro_rules! binary_operator_type {
    ($op:ident, $func:ident, $variant:ident, $lhs:ty, $rhs:ty) => {
        impl ops::$op<$rhs> for $lhs {
            type Output = Expression;
            fn $func(self, rhs: $rhs) -> Expression {
                Expression::$variant(Box::new((self.into(), rhs.into())))
            }
        }
    };
}

macro_rules! binary_operator {
    ($op:ident, $func:ident, $variant:ident) => {
        binary_operator_type!($op, $func, $variant, Expression, Expression);
        binary_operator_type!($op, $func, $variant, Expression, Variable);
        binary_operator_type!($op, $func, $variant, Expression, i64);
        binary_operator_type!($op, $func, $variant, Variable, Expression);
        binary_operator_type!($op, $func, $variant, Variable, Variable);
        binary_operator_type!($op, $func, $variant, Variable, i64);
        binary_operator_type!($op, $func, $variant, i64, Expression);
        binary_operator_type!($op, $func, $variant, i64, Variable);
    };
}

binary_operator!(Add, add, Add);
binary_operator!(Sub, sub, Sub);
binary_operator!(Mul, mul, Mul);
binary_operator!(Div, div, Div);
binary_operator!(Rem, rem, Mod);
