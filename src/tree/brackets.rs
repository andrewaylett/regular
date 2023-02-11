use crate::tokens::Token;
use anyhow::anyhow;

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

struct BracketType(Bracket, StartEnd);

impl TryFrom<Token> for BracketType {
    type Error = anyhow::Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        if let Token::Raw(c) = value {
            match c {
                '(' => return Ok(BracketType(Bracket::Parentheses, StartEnd::Start)),
                '[' => return Ok(BracketType(Bracket::Square, StartEnd::Start)),
                '{' => return Ok(BracketType(Bracket::Braces, StartEnd::Start)),
                ')' => return Ok(BracketType(Bracket::Parentheses, StartEnd::End)),
                ']' => return Ok(BracketType(Bracket::Square, StartEnd::End)),
                '}' => return Ok(BracketType(Bracket::Braces, StartEnd::End)),
                _ => {}
            }
        }
        Err(anyhow!("{value:?} is not a bracket"))
    }
}

impl TryFrom<Token> for Bracket {
    type Error = anyhow::Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        Ok(BracketType::try_from(value)?.0)
    }
}

impl TryFrom<Token> for StartEnd {
    type Error = anyhow::Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        Ok(BracketType::try_from(value)?.1)
    }
}
