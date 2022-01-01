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

    pub fn children(&self) -> ChildIterator {
        use Expression::*;
        let children = match self {
            Not(boxed) => vec![&**boxed].into_iter(),

            Add(boxed) | Sub(boxed) | Mul(boxed) | Div(boxed) | Mod(boxed)
            | Equal(boxed) => vec![&boxed.0, &boxed.1].into_iter(),

            IfThenElse(boxed) => vec![&boxed.0, &boxed.1, &boxed.2].into_iter(),
            _ => vec![].into_iter(),
        };

        ChildIterator { children }
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

    pub fn preorder_iter(&self) -> PreorderExpressionIterator {
        PreorderExpressionIterator {
            root: Some(self),
            stack: Vec::new(),
        }
    }
}

pub struct ChildIterator<'a> {
    children: <Vec<&'a Expression> as IntoIterator>::IntoIter,
}

pub struct PreorderExpressionIterator<'a> {
    root: Option<&'a Expression>,
    stack: Vec<ChildIterator<'a>>,
}

impl<'a> Iterator for ChildIterator<'a> {
    type Item = &'a Expression;
    fn next(&mut self) -> Option<Self::Item> {
        self.children.next()
    }
}

impl<'a> Iterator for PreorderExpressionIterator<'a> {
    type Item = &'a Expression;
    fn next(&mut self) -> Option<Self::Item> {
        // Root node needs to be handled separately, because it
        // wouldn't appear as any child iterator.
        if let Some(root) = self.root.take() {
            self.stack.push(root.children());
            return Some(root);
        }

        // Pop items off the stack until one of them gives us another
        // node.
        while self.stack.len() > 0 {
            let mut iter = self.stack.pop().unwrap();
            let next = iter.next();
            if let Some(item) = next {
                self.stack.push(iter);
                self.stack.push(item.children());
                return Some(item);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_visit() {
        let x: Expression = Variable::new().into();
        let y: Expression = Variable::new().into();
        let expr: Expression =
            3i64 * y.clone() + x.clone().equal_value(y.clone());

        let mut visited: Vec<Expression> = Vec::new();

        expr.walk(|node| {
            visited.push(node.clone());
            match node {
                Expression::Equal(_) => false,
                _ => true,
            }
        });

        let expected = vec![
            3i64 * y.clone() + x.clone().equal_value(y.clone()),
            3i64 * y.clone(),
            3i64.into(),
            y.clone(),
            x.clone().equal_value(y.clone()),
        ];
        assert_eq!(visited.len(), expected.len());
        visited
            .into_iter()
            .zip(expected.into_iter())
            .for_each(|(a, b)| assert_eq!(a, b));
    }

    #[test]
    fn test_preorder_iter() {
        let x: Expression = Variable::new().into();
        let y: Expression = Variable::new().into();
        let expr: Expression = 3i64 * y.clone() + x.clone();

        let visited: Vec<_> = expr.preorder_iter().collect();

        let expected = vec![
            3i64 * y.clone() + x.clone(),
            3i64 * y.clone(),
            3i64.into(),
            y.clone(),
            x.clone(),
        ];
        assert_eq!(visited.len(), expected.len());
        visited
            .into_iter()
            .zip(expected.into_iter())
            .for_each(|(a, b)| assert_eq!(a, &b));
    }
}
