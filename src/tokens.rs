use std::str::Chars;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub(crate) enum Token {
    Escaped(char),
    Raw(char),
    TrailingEscapeCharacter,
}

#[derive(Debug)]
pub(crate) struct TokenIterator<'a>(Chars<'a>);

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| match c {
            '\\' => {
                if let Some(n) = self.0.next() {
                    Token::Escaped(n)
                } else {
                    Token::TrailingEscapeCharacter
                }
            }
            _ => Token::Raw(c),
        })
    }
}

pub(crate) trait Tokenise<'a> {
    fn tokenise(self) -> TokenIterator<'a>;
}

impl<'a> Tokenise<'a> for Chars<'a> {
    fn tokenise(self) -> TokenIterator<'a> {
        TokenIterator(self)
    }
}

impl<'a> Tokenise<'a> for &'a String {
    fn tokenise(self) -> TokenIterator<'a> {
        TokenIterator(self.chars())
    }
}

#[cfg(test)]
mod test {
    use crate::tokens::{Token, Tokenise};
    use rstest::rstest;

    #[rstest]
    #[case("a", vec![Token::Raw('a')])]
    #[case("a\\b", vec![Token::Raw('a'), Token::Escaped('b')])]
    #[case("a\\", vec![Token::Raw('a'), Token::TrailingEscapeCharacter])]
    fn tokenise(#[case] input: String, #[case] expected: Vec<Token>) {
        let actual: Vec<_> = input.tokenise().collect();
        assert_eq!(expected, actual);
    }
}
