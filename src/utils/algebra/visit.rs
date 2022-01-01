use super::Expression;

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
}
