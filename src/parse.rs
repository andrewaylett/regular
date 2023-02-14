use anyhow::{anyhow, Result};

use crate::expression::{ControlFlow, Expression, Literal};
use crate::tree::brackets::Bracket;
use crate::tree::special::Special;
use crate::tree::Node;

pub(crate) fn parse(node: &Node) -> Result<Box<dyn Expression>> {
    match node {
        Node::Empty => Ok(Box::new(Literal("".to_string()))),
        Node::Sequence(sequence) => Ok(Box::new(ControlFlow::Sequence(
            sequence.iter().map(parse).collect::<Result<Vec<_>>>()?,
        ))),
        Node::Tokens(tokens) => Ok(Box::new(Literal(
            tokens.iter().map(|t| t.to_char()).collect(),
        ))),
        Node::Bracketed(bracket_type, content) => match bracket_type {
            Bracket::Parentheses => parse(content),
            Bracket::Braces => {
                unimplemented!()
            }
            Bracket::Square => {
                unimplemented!()
            }
        },
        Node::Special(_) => {
            unimplemented!()
        }
        Node::Augmented(content, augmentation) => {
            let content = parse(content)?;
            if let Node::Special(Special::Star) = **augmentation {
                Ok(Box::new(ControlFlow::Star(content)))
            } else {
                Err(anyhow!("Unimplemented {augmentation:?}"))
            }
        }
        Node::Alternate(a, b) => {
            let mut nodes_to_convert = vec![a, b];
            let mut alternates = vec![];
            while let Some(node) = nodes_to_convert.pop() {
                if let Node::Alternate(x, y) = &**node {
                    nodes_to_convert.push(x);
                    nodes_to_convert.push(y);
                } else {
                    alternates.push(parse(node)?);
                }
            }
            Ok(Box::new(ControlFlow::Alternate(alternates)))
        }
    }
}
