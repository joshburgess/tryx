#![feature(try_trait_v2)]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::string::{String, ToString};
use core::convert::Infallible;
use core::fmt;

use tryx_core::{ControlFlow, FromResidual, Try, TryxResidual};

/// Parser outcome carrying the parsed value and remaining input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsing<'a, T> {
    Done { value: T, rest: &'a str },
    Failed(ParseFailure<'a>),
}

impl<'a, T> Parsing<'a, T> {
    /// Transform a parsed value.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Parsing<'a, U> {
        match self {
            Self::Done { value, rest } => Parsing::Done {
                value: f(value),
                rest,
            },
            Self::Failed(failure) => Parsing::Failed(failure),
        }
    }

    /// Chain another parser after a successful parse.
    pub fn and_then<U>(self, f: impl FnOnce(T, &'a str) -> Parsing<'a, U>) -> Parsing<'a, U> {
        match self {
            Self::Done { value, rest } => f(value, rest),
            Self::Failed(failure) => Parsing::Failed(failure),
        }
    }
}

/// Borrowed parse failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseFailure<'a> {
    pub position: usize,
    pub expected: &'static str,
    pub found: &'a str,
}

impl<'a> ParseFailure<'a> {
    /// Convert the borrowed failure into an owned error.
    pub fn to_error(&self) -> ParseError {
        ParseError {
            position: self.position,
            expected: self.expected,
            found: self.found.to_string(),
        }
    }

    /// Return a display wrapper that renders this failure against `source`.
    pub fn display_with_source<'s>(&'s self, source: &'s str) -> ParseDisplay<'s, 'a> {
        ParseDisplay {
            failure: self,
            source,
        }
    }
}

impl TryxResidual for ParseFailure<'_> {}

/// Owned parse error for escaping borrowed parser lifetimes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub position: usize,
    pub expected: &'static str,
    pub found: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "expected {} at byte {}, found {:?}",
            self.expected, self.position, self.found
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}

pub struct ParseDisplay<'s, 'a> {
    failure: &'s ParseFailure<'a>,
    source: &'s str,
}

impl fmt::Display for ParseDisplay<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.source)?;
        for _ in 0..self.failure.position {
            f.write_str(" ")?;
        }
        writeln!(f, "^ expected {}", self.failure.expected)
    }
}

impl<'a, T> Try for Parsing<'a, T> {
    type Output = (T, &'a str);
    type Residual = ParseFailure<'a>;

    fn from_output((value, rest): Self::Output) -> Self {
        Self::Done { value, rest }
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Done { value, rest } => ControlFlow::Continue((value, rest)),
            Self::Failed(failure) => ControlFlow::Break(failure),
        }
    }
}

impl<'a, T> FromResidual<ParseFailure<'a>> for Parsing<'a, T> {
    fn from_residual(residual: ParseFailure<'a>) -> Self {
        Self::Failed(residual)
    }
}

impl<'a, T> FromResidual<Result<Infallible, ParseFailure<'a>>> for Parsing<'a, T> {
    fn from_residual(residual: Result<Infallible, ParseFailure<'a>>) -> Self {
        match residual {
            Err(failure) => Self::Failed(failure),
        }
    }
}

impl<'a, T, E> FromResidual<ParseFailure<'a>> for Result<T, E>
where
    E: From<ParseError>,
{
    fn from_residual(residual: ParseFailure<'a>) -> Self {
        Err(residual.to_error().into())
    }
}

pub fn literal(input: &str, expected: char) -> Parsing<'_, char> {
    match input.chars().next() {
        Some(found) if found == expected => Parsing::Done {
            value: found,
            rest: &input[found.len_utf8()..],
        },
        _ => Parsing::Failed(failure(input, 0, "literal", input)),
    }
}

pub fn take_while(input: &str, pred: impl Fn(char) -> bool) -> Parsing<'_, &str> {
    let end = input
        .char_indices()
        .find_map(|(index, ch)| (!pred(ch)).then_some(index))
        .unwrap_or(input.len());

    Parsing::Done {
        value: &input[..end],
        rest: &input[end..],
    }
}

pub fn ident(input: &str) -> Parsing<'_, &str> {
    let first = match input.chars().next() {
        Some(ch) if ch == '_' || ch.is_ascii_alphabetic() => ch.len_utf8(),
        _ => return Parsing::Failed(failure(input, 0, "identifier", input)),
    };

    let rest_end = input[first..]
        .char_indices()
        .find_map(|(index, ch)| (!(ch == '_' || ch.is_ascii_alphanumeric())).then_some(index))
        .map_or(input.len(), |index| first + index);

    Parsing::Done {
        value: &input[..rest_end],
        rest: &input[rest_end..],
    }
}

pub fn quoted_string(input: &str) -> Parsing<'_, &str> {
    let Some(after_open) = input.strip_prefix('"') else {
        return Parsing::Failed(failure(input, 0, "quoted string", input));
    };

    match after_open.find('"') {
        Some(end) => Parsing::Done {
            value: &after_open[..end],
            rest: &after_open[end + 1..],
        },
        None => Parsing::Failed(failure(input, input.len(), "closing quote", "")),
    }
}

pub fn eof(input: &str) -> Parsing<'_, ()> {
    if input.is_empty() {
        Parsing::Done {
            value: (),
            rest: input,
        }
    } else {
        Parsing::Failed(failure(input, 0, "end of input", input))
    }
}

fn failure<'a>(
    _source: &'a str,
    position: usize,
    expected: &'static str,
    found: &'a str,
) -> ParseFailure<'a> {
    ParseFailure {
        position,
        expected,
        found: &found[..found.len().min(16)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn parses_key_value() -> Result<(), ParseError> {
        let input = "name=\"tryx\"";
        let (key, rest) = ident(input)?;
        let (_, rest) = literal(rest, '=')?;
        let (value, rest) = quoted_string(rest)?;
        let (_, _) = eof(rest)?;

        assert_eq!((key, value), ("name", "tryx"));
        Ok(())
    }

    #[test]
    fn result_interop_owns_failure() {
        fn parse(input: &str) -> Result<(), ParseError> {
            let (_, rest) = ident(input)?;
            let (_, _) = eof(rest)?;
            Ok(())
        }

        let error = parse("name=").unwrap_err();
        assert_eq!(error.found, "=");
    }

    #[test]
    fn diagnostic_snapshot() {
        let failure = match literal("abc", '=') {
            Parsing::Failed(failure) => failure,
            Parsing::Done { .. } => unreachable!(),
        };

        insta::assert_snapshot!(format!("{}", failure.display_with_source("abc")), @r"
        abc
        ^ expected literal
        ");
    }
}
