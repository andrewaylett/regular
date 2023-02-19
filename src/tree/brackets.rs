use thiserror::Error;

use crate::tokens::Token;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Bracket {
    Parentheses,
    Braces,
    Square,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum StartEnd {
    Start,
    End,
}

#[derive(Debug, Error)]
#[error("{0} is not a bracket at position {}", .0.position())]
pub(crate) struct BracketError(Token);

struct BracketType(Bracket, StartEnd);

impl TryFrom<Token> for BracketType {
    type Error = BracketError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Raw('(', _) => Ok(BracketType(Bracket::Parentheses, StartEnd::Start)),
            Token::Raw('[', _) => Ok(BracketType(Bracket::Square, StartEnd::Start)),
            Token::Raw('{', _) => Ok(BracketType(Bracket::Braces, StartEnd::Start)),
            Token::Raw(')', _) => Ok(BracketType(Bracket::Parentheses, StartEnd::End)),
            Token::Raw(']', _) => Ok(BracketType(Bracket::Square, StartEnd::End)),
            Token::Raw('}', _) => Ok(BracketType(Bracket::Braces, StartEnd::End)),
            _ => Err(BracketError(value)),
        }
    }
}

impl TryFrom<Token> for Bracket {
    type Error = BracketError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        Ok(BracketType::try_from(value)?.0)
    }
}

impl TryFrom<Token> for StartEnd {
    type Error = BracketError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        Ok(BracketType::try_from(value)?.1)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::tokens::{Token, TokenMeta};
    use crate::tree::brackets::Bracket;

    #[rstest]
    fn test_error() {
        let token = Token::Raw('x', TokenMeta { position: 3 });
        let bracket = Bracket::try_from(token);
        let bracket = bracket.map_err(|e| {
            assert_eq!(format!("{e}"), "x is not a bracket at position 3");
            e
        });
        assert!(bracket.is_err())
    }
}
