use crate::tokens::Token;
use anyhow::anyhow;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Special {
    Star,
    Question,
}

impl TryFrom<Token> for Special {
    type Error = anyhow::Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Raw('*') => Ok(Special::Star),
            Token::Raw('?') => Ok(Special::Question),
            _ => Err(anyhow!("{value:?} is not a special character")),
        }
    }
}
