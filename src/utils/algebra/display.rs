use super::{Expression, Variable};

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

impl Expression {
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

// Helper struct for formatting expressions with variable names.
struct NamedExpression<'a, 'b> {
    expr: &'a Expression,
    names: &'b HashMap<Variable, String>,
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
