use anyhow::anyhow;

use crate::tokens::Token;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Special {
    Star,
    Question,
    Caret,
    Dollar,
    Dot,
}

impl TryFrom<Token> for Special {
    type Error = anyhow::Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Raw('*', ..) => Ok(Special::Star),
            Token::Raw('?', ..) => Ok(Special::Question),
            Token::Raw('^', ..) => Ok(Special::Caret),
            Token::Raw('$', ..) => Ok(Special::Dollar),
            Token::Raw('.', ..) => Ok(Special::Dot),
            _ => Err(anyhow!("{value:?} is not a special character")),
        }
    }
}
