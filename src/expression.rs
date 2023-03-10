use std::fmt::Debug;

use itertools::Itertools;

trait ClonableStringIterator: Iterator + Clone {}

pub(crate) trait Expression: Debug {
    fn example(&self) -> String;
    fn enumerate(&self) -> Box<dyn Iterator<Item = String> + '_>;
}

#[derive(Debug)]
pub(crate) struct Literal(pub(crate) String);

impl Expression for Literal {
    fn example(&self) -> String {
        self.0.to_string()
    }

    fn enumerate(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(vec![self.example()].into_iter())
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum ControlFlow {
    Alternate(Vec<Box<dyn Expression>>),
    Optional(Box<dyn Expression>),
    Star(Box<dyn Expression>),
    Sequence(Vec<Box<dyn Expression>>),
}

impl Expression for ControlFlow {
    fn example(&self) -> String {
        match self {
            ControlFlow::Alternate(v) => v[0].example(),
            ControlFlow::Optional(e) => e.example(),
            ControlFlow::Star(e) => e.example(),
            ControlFlow::Sequence(v) => v.iter().map(|e| e.example()).collect(),
        }
    }

    fn enumerate(&self) -> Box<dyn Iterator<Item = String> + '_> {
        return match self {
            ControlFlow::Alternate(v) => Box::new(v.iter().flat_map(|e| e.enumerate())),
            ControlFlow::Optional(e) => Box::new(e.enumerate().chain(["".to_string()])),
            ControlFlow::Star(e) => Box::new(
                ["".to_string()].into_iter().chain(e.enumerate()).chain(
                    e.enumerate()
                        .cartesian_product(e.enumerate().collect::<Vec<_>>())
                        .map(|(a, b)| format!("{a}{b}")),
                ),
            ),
            ControlFlow::Sequence(v) => v.iter().fold(
                Box::new(vec!["".to_string()].into_iter()),
                |prev: Box<dyn Iterator<Item = String>>, expr| {
                    Box::new(
                        prev.cartesian_product(expr.enumerate().collect::<Vec<_>>())
                            .map(|(a, b)| format!("{a}{b}")),
                    )
                },
            ),
        };
    }
}
