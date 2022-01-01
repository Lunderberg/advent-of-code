use super::Expression;

// pub struct VisitorBuilder<Pre, FilterChildren, Leaf, Post> {
//     pre_visit: Option<Pre>,
//     filter_children: Option<FilterChildren>,
//     leaf_visit: Option<Leaf>,
//     post_visit: Option<Post>,
// }

// impl<Pre, FilterChildren, Leaf, Post>
//     VisitorBuilder<Pre, FilterChildren, Leaf, Post>
// where
//     Pre: FnMut(&Expression),
//     Leaf: FnMut(&Expression),
//     Post: FnMut(&Expression),
//     FilterChildren: FnMut(&Expression) -> bool,
// {
//     fn new() -> Self {
//         Self {
//             pre_visit: None,
//             filter_children: None,
//             leaf_visit: None,
//             post_visit: None,
//         }
//     }

//     fn pre_visit(mut self, func: Pre) -> Self {
//         self.pre_visit = Some(func);
//         self
//     }

//     fn filter_children(mut self, func: FilterChildren) -> Self {
//         self.filter_children = Some(func);
//         self
//     }
// }

// struct Visitor {}

// pub trait Visitor {
//     // Pre-visit of nodes with children.  If the return value is true,
//     // the expression's children will be visited.  If the return value
//     // is false, none of the children will be visited, nor will
//     // post_visit be called for the expression.
//     fn pre_visit(&mut self, expr: &Expression) -> bool;

//     // Visit of nodes without children.
//     fn leaf_visit(&mut self, expr: &Expression);

//     // Pre-visit of nodes with children
//     fn post_visit(&mut self, expr: &Expression);

//     // Walk through an expression
//     fn walk(&mut self, expr: &Expression) {
//         let stack = vec![Traversal::PreVisit(expr)];
//         while stack.len() > 0 {}
//     }
// }

#[derive(Debug, PartialEq, Eq)]
pub enum Traversal<'a> {
    PreVisit(&'a Expression),
    LeafVisit(&'a Expression),
    PostVisit(&'a Expression),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TraversalMut<'a> {
    PreVisit(&'a mut Expression),
    LeafVisit(&'a mut Expression),
    PostVisit(&'a mut Expression),
}

pub struct ChildIterator<'a> {
    children: <Vec<&'a Expression> as IntoIterator>::IntoIter,
}

pub struct ExpressionIterator<'a> {
    stack: Vec<Traversal<'a>>,
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

    pub fn iter(&self) -> ExpressionIterator {
        ExpressionIterator {
            stack: vec![self.as_traversal()],
        }
    }

    fn as_traversal(&self) -> Traversal<'_> {
        if self.is_leaf() {
            Traversal::LeafVisit(self)
        } else {
            Traversal::PreVisit(self)
        }
    }

    pub fn preorder_iter(&self) -> impl Iterator<Item = &Expression> + '_ {
        self.iter().filter_map(|node| match node {
            Traversal::PreVisit(node) => Some(node),
            Traversal::LeafVisit(node) => Some(node),
            Traversal::PostVisit(_) => None,
        })
    }

    pub fn postorder_iter(&self) -> impl Iterator<Item = &Expression> + '_ {
        self.iter().filter_map(|node| match node {
            Traversal::PreVisit(_) => None,
            Traversal::LeafVisit(node) => Some(node),
            Traversal::PostVisit(node) => Some(node),
        })
    }
}

impl<'a> Iterator for ChildIterator<'a> {
    type Item = &'a Expression;
    fn next(&mut self) -> Option<Self::Item> {
        self.children.next()
    }
}

impl<'a> DoubleEndedIterator for ChildIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.children.next_back()
    }
}

impl<'a> Iterator for ExpressionIterator<'a> {
    type Item = Traversal<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        use Traversal::*;

        let res = self.stack.pop();

        if let Some(PreVisit(node)) = res {
            self.stack.push(PostVisit(node));
            node.children()
                .rev()
                .map(|child| child.as_traversal())
                .for_each(|visit| self.stack.push(visit));
        }

        res
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

    #[test]
    fn test_postorder_iter() {
        let x: Expression = Variable::new().into();
        let y: Expression = Variable::new().into();
        let expr: Expression = 3i64 * y.clone() + x.clone();

        let visited: Vec<_> = expr.postorder_iter().collect();

        let expected = vec![
            3i64.into(),
            y.clone(),
            3i64 * y.clone(),
            x.clone(),
            3i64 * y.clone() + x.clone(),
        ];
        println!("Visited: {:?}", visited);
        println!("Expected: {:?}", expected);
        assert_eq!(visited.len(), expected.len());
        visited
            .into_iter()
            .zip(expected.into_iter())
            .for_each(|(a, b)| assert_eq!(a, &b));
    }

    #[test]
    fn test_iter() {
        let x: Expression = Variable::new().into();
        let y: Expression = Variable::new().into();
        let expr: Expression = 3i64 * y.clone() + x.clone();

        let visited: Vec<_> = expr.iter().collect();

        use super::Traversal::*;

        let nodes = vec![
            3i64 * y.clone() + x.clone(),
            3i64 * y.clone(),
            3i64.into(),
            y.clone(),
            x.clone(),
        ];

        let expected = vec![
            PreVisit(&nodes[0]),
            PreVisit(&nodes[1]),
            LeafVisit(&nodes[2]),
            LeafVisit(&nodes[3]),
            PostVisit(&nodes[1]),
            LeafVisit(&nodes[4]),
            PostVisit(&nodes[0]),
        ];
        assert_eq!(visited.len(), expected.len());
        visited
            .into_iter()
            .zip(expected.into_iter())
            .for_each(|(a, b)| assert_eq!(a, b));
    }
}
