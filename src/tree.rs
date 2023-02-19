use std::mem::take;

use anyhow::{anyhow, Result};

use crate::tokens::Token;
use crate::tree::brackets::Bracket;
use crate::tree::classify::{Classify, TokenClass};
use crate::tree::special::Special;

pub(crate) mod brackets;
mod classify;
pub(crate) mod special;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub(crate) enum Node {
    #[default]
    Empty,
    Sequence(Vec<Node>),
    Tokens(Vec<Token>),
    Bracketed(Bracket, Box<Node>),
    Special(Special),
    Augmented(Box<Node>, Box<Node>),
    Alternate(Box<Node>, Box<Node>),
}

#[derive(Debug, Clone, Default)]
enum PartialNode {
    #[default]
    Empty,
    Sequence(Vec<Node>),
    Tokens(Vec<Token>),
    Bracketed(Bracket),
    Alternate(Box<Node>),
}

impl PartialNode {
    fn end_with(self, terminator: Node) -> Result<Node> {
        match self {
            PartialNode::Empty => Ok(terminator),
            PartialNode::Sequence(mut sequence) => {
                if let Node::Sequence(_) = terminator {
                    Err(anyhow!("Can't nest sequences"))
                } else {
                    if terminator != Node::Empty {
                        sequence.push(terminator);
                    }
                    if let Some((first, rest)) = sequence.split_first_mut() {
                        if rest.is_empty() {
                            Ok(take(first))
                        } else {
                            Ok(Node::Sequence(sequence))
                        }
                    } else {
                        Ok(Node::Empty)
                    }
                }
            }
            PartialNode::Tokens(t) => {
                if Node::Empty == terminator {
                    Ok(Node::Tokens(t))
                } else {
                    Err(anyhow!("Can't nest {terminator:?} inside a Tokens"))
                }
            }
            PartialNode::Bracketed(b) => Err(anyhow!("Found an unclosed bracket: {b:?}")),
            PartialNode::Alternate(first) => Ok(Node::Alternate(first, Box::new(terminator))),
        }
    }
}

trait AppendChild {
    fn append_child(&mut self, new: Node) -> Result<()>;
}

impl AppendChild for Vec<PartialNode> {
    fn append_child(&mut self, new: Node) -> Result<()> {
        trace!("Append Child, stack: {self:?}, child: {new:?}");
        if let Some(mut containing_partial) = self.pop() {
            match &mut containing_partial {
                PartialNode::Empty => self.push(PartialNode::Sequence(vec![new])),
                PartialNode::Sequence(sequence) => {
                    sequence.push(new);
                    self.push(containing_partial);
                }
                PartialNode::Tokens(tokens) => {
                    self.push(PartialNode::Sequence(vec![Node::Tokens(take(tokens)), new]))
                }
                PartialNode::Bracketed(_) => {
                    self.push(containing_partial);
                    self.push(PartialNode::Sequence(vec![new]));
                }
                PartialNode::Alternate(first) => {
                    self.append_child(Node::Alternate(take(first), Box::new(new)))?;
                }
            }
        } else {
            self.push(PartialNode::Sequence(vec![new]))
        }

        trace!("Appended Child, stack: {self:?}");
        Ok(())
    }
}

fn close_bracket(
    stack: Vec<PartialNode>,
    closing_bracket: Bracket,
    last_node: Node,
) -> Result<Vec<PartialNode>> {
    trace!("Stack: {stack:?}, closing_bracket: {closing_bracket:?}, last_node: {last_node:?}");
    let (stack, remaining_node) = stack.into_iter().try_rfold(
        (None, last_node),
        |(maybe_stack, last_node): (Option<Vec<PartialNode>>, Node), partial| -> Result<_> {
            trace!("Stack: {maybe_stack:?}, Last Node: {last_node:?}");
            if let Some(mut stack) = maybe_stack {
                stack.insert(0, partial);
                if last_node != Node::Empty {
                    stack.append_child(last_node)?
                }
                Ok((Some(stack), Node::Empty))
            } else if let PartialNode::Bracketed(opening_bracket) = partial {
                if opening_bracket == closing_bracket {
                    Ok((
                        Some(vec![]),
                        Node::Bracketed(opening_bracket, Box::new(last_node)),
                    ))
                } else {
                    Err(anyhow!(
                        "Mismatched brackets: {opening_bracket:?} closed by {closing_bracket:?}"
                    ))
                }
            } else {
                let next_node = partial.end_with(last_node)?;
                Ok((None, next_node))
            }
        },
    )?;
    let stack = stack.ok_or(anyhow!("Close Bracket without Opening Bracket"))?;

    trace!("Stack: {stack:?} Remaining Node: {remaining_node:?}");
    if remaining_node == Node::Empty {
        Ok(stack)
    } else {
        assert!(stack.is_empty());
        Ok(vec![PartialNode::Sequence(vec![remaining_node])])
    }
}

pub(crate) fn tree<T: IntoIterator<Item = Token>>(tokens: T) -> Result<Node> {
    let stack: Vec<PartialNode> = tokens.into_iter().try_fold(
        vec![PartialNode::Empty],
        |mut stack: Vec<PartialNode>, token: Token| -> Result<Vec<PartialNode>> {
            trace!("Token: {token:?}, Stack: {stack:?}");
            let token_class = token.classify();
            if let Some(node) = stack.pop() {
                match node {
                    PartialNode::Empty => match token_class {
                        TokenClass::OpenBracket => {
                            stack.push(PartialNode::Bracketed(Bracket::try_from(token)?));
                            stack.push(PartialNode::Empty);
                            Ok(stack)
                        }
                        TokenClass::CloseBracket => {
                            close_bracket(stack, token.try_into()?, Node::Empty)
                        }
                        TokenClass::Augmentation => {
                            stack.push(PartialNode::Sequence(vec![Node::Special(
                                token.try_into()?,
                            )]));
                            Ok(stack)
                        }
                        TokenClass::Regular => {
                            stack.push(PartialNode::Tokens(vec![token]));
                            Ok(stack)
                        }
                        TokenClass::Problematic => return Err(anyhow!("Unexpected escape")),
                        TokenClass::Alternation => {
                            stack.push(PartialNode::Alternate(Box::new(Node::Empty)));
                            stack.push(PartialNode::Empty);
                            Ok(stack)
                        }
                    },
                    PartialNode::Sequence(mut sequence) => match token_class {
                        TokenClass::OpenBracket => {
                            stack.push(PartialNode::Sequence(sequence));
                            stack.push(PartialNode::Bracketed(token.try_into()?));
                            stack.push(PartialNode::Empty);
                            Ok(stack)
                        }
                        TokenClass::CloseBracket => close_bracket(
                            stack,
                            token.try_into()?,
                            PartialNode::Sequence(sequence).end_with(Node::Empty)?,
                        ),
                        TokenClass::Augmentation => {
                            let previous = sequence
                                .pop()
                                .ok_or(anyhow!("Special character at start of sequence"))?;
                            sequence.push(Node::Augmented(
                                Box::new(previous),
                                Box::new(Node::Special(token.try_into()?)),
                            ));
                            stack.push(PartialNode::Sequence(sequence));
                            Ok(stack)
                        }
                        TokenClass::Regular => {
                            stack.push(PartialNode::Sequence(sequence));
                            stack.push(PartialNode::Tokens(vec![token]));
                            Ok(stack)
                        }
                        TokenClass::Problematic => Err(anyhow!("Unexpected Escape")),
                        TokenClass::Alternation => {
                            stack.push(PartialNode::Alternate(Box::new(Node::Sequence(sequence))));
                            stack.push(PartialNode::Empty);
                            Ok(stack)
                        }
                    },
                    PartialNode::Tokens(mut token_sequence) => match token_class {
                        TokenClass::OpenBracket => {
                            let container = stack.pop().ok_or(anyhow!("Stack Underflow"))?;
                            match container {
                                PartialNode::Empty => {
                                    return Err(anyhow!("Found an unexpected Empty on the stack"))
                                }
                                PartialNode::Sequence(mut sequence) => {
                                    sequence.push(Node::Tokens(token_sequence));
                                    stack.push(PartialNode::Sequence(sequence))
                                }
                                PartialNode::Tokens(_) => {
                                    return Err(anyhow!("Unexpected nested token sequences"))
                                }
                                PartialNode::Bracketed(_) | PartialNode::Alternate(_) => {
                                    stack.push(container);
                                    stack.push(PartialNode::Sequence(vec![Node::Tokens(
                                        token_sequence,
                                    )]));
                                }
                            }
                            stack.push(PartialNode::Bracketed(token.try_into()?));
                            stack.push(PartialNode::Empty);
                            Ok(stack)
                        }
                        TokenClass::CloseBracket => {
                            close_bracket(stack, token.try_into()?, Node::Tokens(token_sequence))
                        }
                        TokenClass::Augmentation => {
                            if let Some(augmentee) = token_sequence.pop() {
                                stack.append_child(Node::Tokens(token_sequence))?;
                                stack.append_child(Node::Augmented(
                                    Box::new(Node::Tokens(vec![augmentee])),
                                    Box::new(Node::Special(token.try_into()?)),
                                ))?;
                            } else {
                                stack.append_child(Node::Special(token.try_into()?))?;
                            }
                            Ok(stack)
                        }
                        TokenClass::Regular => {
                            token_sequence.push(token);
                            stack.push(PartialNode::Tokens(token_sequence));
                            Ok(stack)
                        }
                        TokenClass::Problematic => Err(anyhow!("Unexpected Escape")),
                        TokenClass::Alternation => {
                            stack.push(PartialNode::Alternate(Box::new(Node::Tokens(
                                token_sequence,
                            ))));
                            stack.push(PartialNode::Empty);
                            Ok(stack)
                        }
                    },
                    PartialNode::Bracketed(_) | PartialNode::Alternate(_) => {
                        Err(anyhow!("Should not find {node:?} at the top of the stack"))
                    }
                }
            } else {
                Err(anyhow!("Ran out of stack"))
            }
        },
    )?;

    assert!(!stack.is_empty());

    stack
        .into_iter()
        .try_rfold(Node::Empty, |node, partial| partial.end_with(node))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod test {
    use lazy_static::lazy_static;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::tokens::Token::*;
    use crate::tokens::{Token, TokenMeta, Tokenise};
    use crate::tree::special::Special::Star;
    use crate::tree::Node::*;
    use crate::tree::{tree, Bracket, Node};

    lazy_static! {
        static ref STAR: Node = Bracketed(
            Bracket::Parentheses,
            Box::new(Augmented(
                Box::new(Bracketed(
                    Bracket::Parentheses,
                    Box::new(Alternate(
                        Box::new(Empty),
                        Box::new(Tokens(vec![Raw('a', TokenMeta { position: 3 })]))
                    ))
                )),
                Box::new(Special(Star))
            ))
        );
    }

    #[rstest]
    #[case("a", &Node::Tokens(vec![Token::Raw('a', TokenMeta {position: 0})]))]
    #[case("ab*", &Sequence(vec![Tokens(vec![Raw('a', TokenMeta {position: 0})]), Augmented(Box::new(Tokens(vec![Raw('b', TokenMeta {position: 1})])), Box::new(Special(Star)))]))]
    #[case("a|", &Alternate(Box::new(Tokens(vec![Raw('a', TokenMeta {position: 0})])), Box::new(Empty)))]
    #[case("|a", &Alternate(Box::new(Empty), Box::new(Tokens(vec![Raw('a', TokenMeta {position: 1})]))))]
    #[case("(|a)", &Bracketed(Bracket::Parentheses, Box::new(Alternate(Box::new(Empty), Box::new(Tokens(vec![Raw('a', TokenMeta {position: 2})]))))))]
    #[case("((|a)*)", &STAR)]
    #[case("(|a\\))", &Bracketed(Bracket::Parentheses, Box::new(Alternate(Box::new(Empty), Box::new(Tokens(vec![Raw('a', TokenMeta {position: 2}),Escaped(')', TokenMeta {position: 3})]))))))]
    #[case("[*]", &Bracketed(Bracket::Square, Box::new(Special(Star))))]
    fn test_tree(#[case] input: String, #[case] expected: &Node) {
        let tokens = input.tokenise();
        let actual = tree(tokens).expect("Failed to tree");
        assert_eq!(expected, &actual);
    }
}
