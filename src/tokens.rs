use std::fmt::{Display, Formatter, Write};
use std::iter::Enumerate;
use std::str::Chars;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub(crate) struct TokenMeta {
    pub(crate) position: usize,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub(crate) enum Token {
    Escaped(char, TokenMeta),
    Raw(char, TokenMeta),
    TrailingEscapeCharacter(TokenMeta),
}

impl Token {
    pub(crate) fn to_char(self) -> char {
        match self {
            Token::Escaped(c, _) => c,
            Token::Raw(c, _) => c,
            Token::TrailingEscapeCharacter(_) => '\\',
        }
    }

    pub(crate) fn position(&self) -> usize {
        let meta = self.meta();
        meta.position
    }

    fn meta(&self) -> &TokenMeta {
        match self {
            Token::Escaped(_, m) => m,
            Token::Raw(_, m) => m,
            Token::TrailingEscapeCharacter(m) => m,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.to_char())
    }
}

#[derive(Debug)]
pub(crate) struct TokenIterator<'a>(Enumerate<Chars<'a>>);

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(position, c)| match c {
            '\\' => {
                if let Some((_, n)) = self.0.next() {
                    Token::Escaped(n, TokenMeta { position })
                } else {
                    Token::TrailingEscapeCharacter(TokenMeta { position })
                }
            }
            _ => Token::Raw(c, TokenMeta { position }),
        })
    }
}

pub(crate) trait Tokenise<'a> {
    fn tokenise(self) -> TokenIterator<'a>;
}

impl<'a> Tokenise<'a> for Chars<'a> {
    fn tokenise(self) -> TokenIterator<'a> {
        TokenIterator(self.enumerate())
    }
}

impl<'a> Tokenise<'a> for &'a String {
    fn tokenise(self) -> TokenIterator<'a> {
        TokenIterator(self.chars().enumerate())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::tokens::{Token, TokenMeta, Tokenise};

    #[rstest]
    #[case("a", vec![Token::Raw('a', TokenMeta {position: 0})])]
    #[case("a\\b", vec![Token::Raw('a', TokenMeta {position: 0}), Token::Escaped('b', TokenMeta {position: 1})])]
    #[case("a\\", vec![Token::Raw('a', TokenMeta {position: 0}), Token::TrailingEscapeCharacter(TokenMeta {position: 1})])]
    fn tokenise(#[case] input: String, #[case] expected: Vec<Token>) {
        let actual: Vec<_> = input.tokenise().collect();
        assert_eq!(expected, actual);
    }
}
