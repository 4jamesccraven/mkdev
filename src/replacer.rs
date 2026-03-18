//! A simple parser and formatter for format-string-like behaviour.
//!
//! This module is used to implement mkdev's recipe substitutions during `mk evoke` as well as
//! formatting recipes for the default `mk list` behaviour.
#![allow(dead_code)]
use std::collections::HashMap;

/// The primary interface for the formatter.
///
/// Takes a set of mappings, the delimiter pairs that indicate a variable, and a strategy for
/// handling undefined variables.
#[derive(Clone, Debug)]
pub struct ReplaceFmt {
    subs: HashMap<String, String>,
    delims: Delimiters,
    strategy: InvalidTokenStrategy,
}

impl ReplaceFmt {
    pub fn new(
        subs: HashMap<String, String>,
        delims: (&str, &str),
        fallback_strategy: InvalidTokenStrategy,
    ) -> Self {
        Self {
            subs,
            delims: Delimiters {
                open: delims.0.chars().collect(),
                close: delims.1.chars().collect(),
            },
            strategy: fallback_strategy,
        }
    }

    /// Replaces all variables in `src` according to the formatter's internal mapping.
    pub fn replace(&self, src: &str) -> String {
        self.replace_with(src, |val| Some(val.to_string()))
    }

    /// Replaces all variables by applying `resolver` to the value in the internal mapping.
    pub fn replace_with<F>(&self, src: &str, resolver: F) -> String
    where
        F: Fn(&str) -> Option<String>,
    {
        let parser = Parser {
            source: src.chars().collect(),
            curr: 0,
            delims: self.delims.clone(),
        };

        let tokens = parser.parse();

        tokens
            .into_iter()
            .filter_map(|token| match token {
                Segment::Text(text) => Some(text),
                Segment::Token(key) => {
                    let def = self.subs.get(&key);
                    match def {
                        // Special case for reserved names.
                        Some(val) if key.starts_with("mk::") => Some(val.to_string()),
                        // User-defined resolution
                        Some(val) => resolver(val),
                        // Strategy-defined resolution for failed lookups.
                        None => match self.strategy {
                            InvalidTokenStrategy::PassThrough => Some(key),
                            InvalidTokenStrategy::Preserve => Some(format!(
                                "{}{}{}",
                                self.delims.open.iter().collect::<String>(),
                                key,
                                self.delims.close.iter().collect::<String>()
                            )),
                            InvalidTokenStrategy::Ignore => None,
                        },
                    }
                }
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

/// A single-use parser for a format string.
///
/// Parses a `Vec<char>` into a `Vec<Segment>` for the `ReplaceFmt` struct to consume. The parser
/// locates special "tokens" which are delimited by the provided delimiters. If an open delimiter
/// is found with a leading backslash ('\'), it is ignored and treated as part of the current text
/// segment.
#[derive(Clone, Debug)]
struct Parser {
    source: Vec<char>,
    curr: usize,
    delims: Delimiters,
}

impl Parser {
    /// Parses out the the Tokens, consuming the parser.
    pub fn parse(mut self) -> Vec<Segment> {
        let mut out = vec![];
        while !self.at_end() {
            out.push(self.next_segment());
        }
        out
    }

    /// Yields the next contiguous segment from the source buffer.
    fn next_segment(&mut self) -> Segment {
        if self.has_open() {
            self.parse_token()
        } else {
            self.parse_text()
        }
    }

    /// Parses out a delimited token, returning the token name and consuming the delimiters.
    fn parse_token(&mut self) -> Segment {
        self.curr += self.delims.open.len();
        let mut buf = String::new();
        let mut unclosed = false;
        loop {
            if self.at_end() {
                let prefix: String = self.delims.open.iter().collect();
                buf.insert_str(0, &prefix);
                unclosed = true;
                break;
            }
            buf.push(self.advance());
            if self.has_close() {
                self.curr += self.delims.close.len();
                break;
            }
        }

        if buf.is_empty() || unclosed {
            Segment::Text(buf)
        } else {
            Segment::Token(buf)
        }
    }

    /// Parses out generic text until the first non-escaped open delimiter is found.
    fn parse_text(&mut self) -> Segment {
        let mut buf = String::new();
        while !self.has_open() && !self.at_end() {
            // General case: pass characters through to buffer
            if !self.has_escaped_open() {
                buf.push(self.advance());
                continue;
            }

            // Escaped delimiters: skip the \, pass the delimiters through
            // untouched.
            _ = self.advance();
            for _ in 0..self.delims.open.len() {
                buf.push(self.advance());
            }
        }
        Segment::Text(buf)
    }

    /// Yields the character at the current place in the buffer and advances.
    fn advance(&mut self) -> char {
        let c = self.source[self.curr];
        self.curr += 1;
        c
    }

    /// Returns `true` if the remainder of the buffer starts with an open delimiter.
    fn has_open(&self) -> bool {
        self.source[self.curr..].starts_with(&self.delims.open)
    }

    /// Returns `true` if the remainder of the buffer starts with an escaped open delimiter.
    fn has_escaped_open(&self) -> bool {
        self.source[self.curr..].starts_with(&['\\'])
            && self
                .source
                .get(self.curr + 1..)
                .is_some_and(|s| s.starts_with(&self.delims.open))
    }

    /// Returns `true` if the remainder of the buffer starts with a close delimiter.
    fn has_close(&self) -> bool {
        self.source[self.curr..].starts_with(&self.delims.close)
    }

    /// Returns `true` if the buffer has been fully read.
    ///
    /// This is determined by checking if the buffer pointer has reached or exceeded the length of
    /// the buffer.
    fn at_end(&self) -> bool {
        self.curr >= self.source.len()
    }
}

#[derive(Clone, Debug)]
struct Delimiters {
    open: Vec<char>,
    close: Vec<char>,
}

#[derive(Clone, Debug)]
enum Segment {
    Token(String),
    Text(String),
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum InvalidTokenStrategy {
    /// Pass just the token name
    PassThrough,
    /// Pass the token name surrounded by its delimiters
    Preserve,
    /// Pass nothing
    Ignore,
}

#[cfg(test)]
mod tests {
    use super::InvalidTokenStrategy::*;
    use super::*;

    fn make_fmt(
        subs: &[(&str, &str)],
        delims: (&str, &str),
        strat: InvalidTokenStrategy,
    ) -> ReplaceFmt {
        ReplaceFmt::new(
            subs.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            delims,
            strat,
        )
    }

    #[test]
    fn replaces_single_token() {
        let fmt = make_fmt(&[("name", "Alice")], ("{{", "}}"), Ignore);
        assert_eq!(fmt.replace("Hello, {{name}}!"), "Hello, Alice!");
    }

    #[test]
    fn replaces_multiple_tokens() {
        let fmt = make_fmt(&[("a", "foo"), ("b", "bar")], ("{{", "}}"), Ignore);
        assert_eq!(fmt.replace("{{a}} and {{b}}"), "foo and bar");
    }

    #[test]
    fn unknown_token_passes_through() {
        let fmt = make_fmt(&[], ("{{", "}}"), PassThrough);
        assert_eq!(fmt.replace("{{unknown}}"), "unknown");
    }

    #[test]
    fn no_tokens_returns_source_unchanged() {
        let fmt = make_fmt(&[("x", "y")], ("{{", "}}"), Ignore);
        assert_eq!(fmt.replace("no tokens here"), "no tokens here");
    }

    #[test]
    fn empty_source_returns_empty() {
        let fmt = make_fmt(&[("x", "y")], ("{{", "}}"), Ignore);
        assert_eq!(fmt.replace(""), "");
    }

    #[test]
    fn custom_delimiters() {
        let fmt = make_fmt(&[("name", "Bob")], ("<", ">"), Ignore);
        assert_eq!(fmt.replace("Hello, <name>!"), "Hello, Bob!");
    }

    #[test]
    fn adjacent_tokens() {
        let fmt = make_fmt(&[("a", "foo"), ("b", "bar")], ("{{", "}}"), Ignore);
        assert_eq!(fmt.replace("{{a}}{{b}}"), "foobar");
    }

    #[test]
    fn token_at_start_and_end() {
        let fmt = make_fmt(&[("x", "!")], ("{{", "}}"), Ignore);
        assert_eq!(fmt.replace("{{x}}hello{{x}}"), "!hello!");
    }

    #[test]
    fn unclosed_delimiter_treated_as_text() {
        let fmt = make_fmt(&[("name", "Alice")], ("{{", "}}"), Ignore);
        let result = fmt.replace("{{name");
        assert!(!result.contains("Alice"));
    }

    #[test]
    fn empty_token_treated_as_text() {
        let fmt = make_fmt(&[("", "oops")], ("{{", "}}"), Ignore);
        assert_eq!(fmt.replace("{{}}"), "{{}}");
    }

    #[test]
    fn ignore_escaped_delimiter() {
        let fmt = make_fmt(&[("a", "b")], ("{{", "}}"), Ignore);
        assert_eq!(fmt.replace("\\{{a}}"), "{{a}}");
    }

    #[test]
    fn regular_escapes_ignored() {
        let fmt = make_fmt(&[("a", "b")], ("{{", "}}"), Ignore);
        assert_eq!(fmt.replace("\\ {{a}}"), "\\ b");
    }

    #[test]
    fn escaped_single_char_delimiter() {
        let fmt = make_fmt(&[("a", "b")], ("{", "}"), Ignore);
        assert_eq!(fmt.replace("\\{a}"), "{a}");
    }

    #[test]
    fn trailing_backslash_does_not_panic() {
        let fmt = make_fmt(&[("a", "b")], ("{{", "}}"), Ignore);
        assert_eq!(fmt.replace("hello\\"), "hello\\");
    }

    #[test]
    fn trailing_backslash_single_char_delimiter_does_not_panic() {
        let fmt = make_fmt(&[("a", "b")], ("{", "}"), Ignore);
        assert_eq!(fmt.replace("hello\\"), "hello\\");
    }
}
