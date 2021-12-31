use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops;
use std::sync::atomic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Variable {
    id: usize,
}

#[derive(Debug, Clone, Hash)]
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

// Helper struct for formatting expressions with variable names.
struct NamedExpression<'a, 'b> {
    expr: &'a Expression,
    names: &'b HashMap<Variable, String>,
}

impl Expression {
    // Walk through the expression tree.  For each node, execute the
    // callback.  If the callback returns true, continue recursing
    // through that branch of the tree.
    pub fn walk<F>(&self, mut f: F)
    where
        F: FnMut(&Expression) -> bool,
    {
        self.walk_impl(&mut f);
    }

    fn walk_impl<F>(&self, f: &mut F)
    where
        F: FnMut(&Expression) -> bool,
    {
        let recurse = f(self);

        if recurse {
            self.children().for_each(|child| child.walk_impl(f))
        }
    }

    pub fn is_leaf(&self) -> bool {
        use Expression::*;
        match self {
            Impossible | Variable(_) | Int(_) => true,
            _ => false,
        }
    }

    pub fn children(&self) -> impl Iterator<Item = &Expression> + '_ {
        use Expression::*;
        match self {
            Not(boxed) => vec![&**boxed].into_iter(),

            Add(boxed) | Sub(boxed) | Mul(boxed) | Div(boxed) | Mod(boxed)
            | Equal(boxed) => vec![&boxed.0, &boxed.1].into_iter(),

            IfThenElse(boxed) => vec![&boxed.0, &boxed.1, &boxed.2].into_iter(),
            _ => vec![].into_iter(),
        }
    }

    // Visit all nodes.  Equivalent to calling walk() with a function
    // that always returns true.
    pub fn visit<F>(&self, mut f: F)
    where
        F: FnMut(&Expression),
    {
        self.walk(|node| {
            f(node);
            true
        })
    }

    pub fn equal_value(self, other: Self) -> Self {
        Expression::Equal(Box::new((self, other)))
    }

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
        let mut out: HashSet<Variable> = HashSet::new();
        self.visit(|expr| {
            if let Expression::Variable(var) = expr {
                out.insert(*var);
            }
        });
        out
    }

    pub fn solve_for(&self, var: Variable) -> Option<Expression> {
        Self::solve_for_impl(var, self.clone(), 1.into())
            .map(|expr| expr.simplify())
        // match self {
        //     Expression::Equal(boxed) => {
        //         let (a, b) = &**boxed;
        //         Self::solve_for_impl(var, a.clone(), b.clone())
        //     }
        //     _ => None,
        // }
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

    pub fn format(&self, names: &HashMap<Variable, String>) -> String {
        format!(
            "{}",
            NamedExpression {
                expr: self,
                names: &names
            }
        )
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

impl ops::Add for Expression {
    type Output = Expression;
    fn add(self, rhs: Self) -> Self {
        use Expression::*;
        match (self, rhs) {
            (Impossible, _) | (_, Impossible) => Impossible,
            (Int(a), Int(b)) => Int(a + b),
            (a, b) => Add(Box::new((a, b))),
        }
    }
}

impl ops::Sub for Expression {
    type Output = Expression;
    fn sub(self, rhs: Self) -> Self {
        use Expression::*;
        match (self, rhs) {
            (Impossible, _) | (_, Impossible) => Impossible,
            (Int(a), Int(b)) => Int(a - b),
            (a, b) => Sub(Box::new((a, b))),
        }
    }
}

impl ops::Mul for Expression {
    type Output = Expression;
    fn mul(self, rhs: Self) -> Self {
        use Expression::*;
        match (self, rhs) {
            (Impossible, _) | (_, Impossible) => Impossible,
            (Int(a), Int(b)) => Int(a * b),
            (a, b) => Mul(Box::new((a, b))),
        }
    }
}

impl ops::Div for Expression {
    type Output = Expression;
    fn div(self, rhs: Self) -> Self {
        use Expression::*;
        match (self, rhs) {
            (Impossible, _) | (_, Impossible) => Impossible,
            (Int(a), Int(b)) => Int(a / b),
            (a, b) => Div(Box::new((a, b))),
        }
    }
}

impl ops::Rem for Expression {
    type Output = Expression;
    fn rem(self, rhs: Self) -> Self {
        use Expression::*;
        match (self, rhs) {
            (Impossible, _) | (_, Impossible) => Impossible,
            (Int(a), Int(b)) => Int(a % b),
            (a, b) => Mod(Box::new((a, b))),
        }
    }
}

impl<'a, 'b> Display for NamedExpression<'a, 'b> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Expression::*;

        let precedence = |node: &Expression| -> usize {
            use Expression::*;
            match node {
                //
                Impossible => 0,
                Variable(_) => 0,
                Int(_) => 0,
                //
                Not(expr) => match &**expr {
                    Equal(_) => 40,
                    _ => 10,
                },
                //
                Div(_) => 15,
                //
                Mul(_) => 20,
                Mod(_) => 20,
                //
                Add(_) => 30,
                Sub(_) => 30,
                //
                Equal(_) => 40,
                //
                IfThenElse(_) => 50,
            }
        };

        let associative = |node: &Expression| -> bool {
            use Expression::*;

            match node {
                Impossible => true,
                Variable(_) => true,
                Int(_) => true,
                Not(expr) => match &**expr {
                    Equal(_) => false,
                    _ => true,
                },
                Div(_) => false,
                Mul(_) => true,
                Mod(_) => false,
                Add(_) => true,
                Sub(_) => false,
                Equal(_) => false,
                IfThenElse(_) => false,
            }
        };

        let format_child = |node: &Expression| {
            let named = NamedExpression {
                expr: node,
                names: self.names,
            };
            let child_precedence = precedence(node);
            let self_precedence = precedence(self.expr);
            if (child_precedence > self_precedence)
                || ((child_precedence == self_precedence)
                    && (!associative(self.expr)))
            {
                format!("({})", named)
            } else {
                format!("{}", named)
            }
        };

        match self.expr {
            Impossible => write!(f, "!!!"),
            Int(num) => write!(f, "{}", num),
            Not(expr) => match &**expr {
                Equal(boxed) => {
                    write!(
                        f,
                        "{} != {}",
                        format_child(&boxed.0),
                        format_child(&boxed.1)
                    )
                }
                _ => write!(f, "!{}", format_child(expr)),
            },
            Variable(var) => {
                if let Some(name) = self.names.get(var) {
                    write!(f, "{}", name)
                } else {
                    write!(f, "{}", var)
                }
            }
            Equal(boxed) => {
                let (a, b) = &**boxed;
                write!(f, "{} == {}", format_child(a), format_child(b))
            }
            Add(boxed) => {
                let (a, b) = &**boxed;
                write!(f, "{} + {}", format_child(a), format_child(b))
            }
            Sub(boxed) => {
                let (a, b) = &**boxed;
                write!(f, "{} - {}", format_child(a), format_child(b))
            }
            Mod(boxed) => {
                let (a, b) = &**boxed;
                write!(f, "{} % {}", format_child(a), format_child(b))
            }
            Mul(boxed) => {
                let (a, b) = &**boxed;
                write!(f, "{} * {}", format_child(a), format_child(b))
            }
            Div(boxed) => {
                let (a, b) = &**boxed;
                write!(f, "{} / {}", format_child(a), format_child(b))
            }
            IfThenElse(boxed) => {
                let (cond, if_expr, then_expr) = &**boxed;
                write!(
                    f,
                    "{} ? {} : {}",
                    format_child(cond),
                    format_child(if_expr),
                    format_child(then_expr),
                )
            }
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let names = HashMap::new();
        write!(
            f,
            "{}",
            NamedExpression {
                expr: self,
                names: &names
            }
        )
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "var{}", self.id)
    }
}
