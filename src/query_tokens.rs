use std::{mem, str};

use QueryToken::{Quoted, Free};

#[derive(Debug, PartialEq, Eq)]
pub enum QueryToken<'a> {
    Free(&'a str),
    Quoted(&'a str),
}

enum State {
    Free(usize),
    Quoted(usize),
    Fused,
}

impl State {
    fn is_quoted(&self) -> bool {
        match self { State::Quoted(_) => true, _ => false }
    }

    fn replace_by(&mut self, state: State) -> State {
        mem::replace(self, state)
    }
}

pub struct QueryTokens<'a> {
    state: State,
    string: &'a str,
    string_chars: str::CharIndices<'a>,
}

impl<'a> QueryTokens<'a> {
    pub fn new(query: &'a str) -> QueryTokens<'a> {
        QueryTokens {
            state: State::Free(0),
            string: query,
            string_chars: query.char_indices(),
        }
    }
}

impl<'a> Iterator for QueryTokens<'a> {
    type Item = QueryToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (i, afteri, c) = match self.string_chars.next() {
                Some((i, c)) => (i, i + c.len_utf8(), c),
                None => return match self.state.replace_by(State::Fused) {
                    State::Free(s) => if !self.string[s..].is_empty() {
                        Some(Free(&self.string[s..]))
                    } else {
                        None
                    },
                    State::Quoted(s) => Some(Quoted(&self.string[s..])),
                    State::Fused => None,
                },
            };

            if c == '"' {
                match self.state.replace_by(State::Free(afteri)) {
                    State::Quoted(s) => return Some(Quoted(&self.string[s..i])),
                    State::Free(s) => {
                        self.state = State::Quoted(afteri);
                        if i > s { return Some(Free(&self.string[s..i])) }
                    },
                    State::Fused => return None,
                }
            }
            else if !self.state.is_quoted() && !c.is_alphanumeric() {
                match self.state.replace_by(State::Free(afteri)) {
                    State::Free(s) if i > s => return Some(Free(&self.string[s..i])),
                    _ => self.state = State::Free(afteri),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_quoted_string() {
        use QueryToken::Quoted;

        let mut iter = QueryTokens::new("\"hello\"");
        assert_eq!(iter.next(), Some(Quoted("hello")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn one_pending_quoted_string() {
        use QueryToken::Quoted;

        let mut iter = QueryTokens::new("\"hello");
        assert_eq!(iter.next(), Some(Quoted("hello")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn one_non_quoted_string() {
        use QueryToken::Free;

        let mut iter = QueryTokens::new("hello");
        assert_eq!(iter.next(), Some(Free("hello")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn quoted_directly_followed_by_free_strings() {
        use QueryToken::{Quoted, Free};

        let mut iter = QueryTokens::new("\"hello\"world");
        assert_eq!(iter.next(), Some(Quoted("hello")));
        assert_eq!(iter.next(), Some(Free("world")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn free_directly_followed_by_quoted_strings() {
        use QueryToken::{Quoted, Free};

        let mut iter = QueryTokens::new("hello\"world\"");
        assert_eq!(iter.next(), Some(Free("hello")));
        assert_eq!(iter.next(), Some(Quoted("world")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn free_followed_by_quoted_strings() {
        use QueryToken::{Quoted, Free};

        let mut iter = QueryTokens::new("hello \"world\"");
        assert_eq!(iter.next(), Some(Free("hello")));
        assert_eq!(iter.next(), Some(Quoted("world")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn multiple_spaces_separated_strings() {
        use QueryToken::Free;

        let mut iter = QueryTokens::new("hello    world   ");
        assert_eq!(iter.next(), Some(Free("hello")));
        assert_eq!(iter.next(), Some(Free("world")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn multi_interleaved_quoted_free_strings() {
        use QueryToken::{Quoted, Free};

        let mut iter = QueryTokens::new("hello \"world\" coucou \"monde\"");
        assert_eq!(iter.next(), Some(Free("hello")));
        assert_eq!(iter.next(), Some(Quoted("world")));
        assert_eq!(iter.next(), Some(Free("coucou")));
        assert_eq!(iter.next(), Some(Quoted("monde")));
        assert_eq!(iter.next(), None);
    }
}
