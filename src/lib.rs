#![deny(
    bad_style,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::expect_used
)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;

use anyhow::{Context, Result};

use crate::expression::Expression;
use crate::tokens::Tokenise;
use crate::tree::tree;

mod expression;
mod parse;
mod tokens;
mod tree;

/// Provides an example from the regular language described by the expression
pub fn example(expression: String) -> Result<()> {
    let example = parse(expression)
        .context("Failed to parse expression")?
        .example();
    println!("{example}");
    Ok(())
}

/// Provides a (non-exhaustive) enumeration of the members of the regular language described by the expression
pub fn enumerate(expression: String) -> Result<()> {
    for example in parse(expression)
        .context("Failed to parse expression")?
        .enumerate()
    {
        println!("{example}")
    }
    Ok(())
}

fn parse(expression: String) -> Result<Box<dyn Expression>> {
    let tokens = expression.tokenise();
    let tree = tree(tokens)?;
    let expression = parse::parse(&tree)?;
    Ok(expression)
}
