use crate::tokens::Token;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum TokenClass {
    OpenBracket,
    CloseBracket,
    Augmentation,
    Alternation,
    Regular,
    Problematic,
}

pub(crate) trait Classify {
    fn classify(&self) -> TokenClass;
}

impl Classify for Token {
    fn classify(&self) -> TokenClass {
        match self {
            Token::Escaped(..) => TokenClass::Regular,
            Token::Raw(c, ..) => match c {
                '(' | '[' | '{' => TokenClass::OpenBracket,
                ')' | ']' | '}' => TokenClass::CloseBracket,
                '*' | '?' => TokenClass::Augmentation,
                '|' => TokenClass::Alternation,
                _ => TokenClass::Regular,
            },
            Token::TrailingEscapeCharacter(..) => TokenClass::Problematic,
        }
    }
}
