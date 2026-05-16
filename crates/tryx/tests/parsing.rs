#![cfg(feature = "parsing")]

use tryx::parsing::{ParseError, eof, ident, literal, quoted_string};

fn parse(input: &str) -> Result<(&str, &str), ParseError> {
    let (key, rest) = ident(input)?;
    let (_, rest) = literal(rest, '=')?;
    let (value, rest) = quoted_string(rest)?;
    let (_, _) = eof(rest)?;
    Ok((key, value))
}

#[test]
fn umbrella_reexports_parsing_api() {
    assert_eq!(parse("name=\"tryx\""), Ok(("name", "tryx")));
}

#[test]
fn result_interop_returns_owned_error() {
    let error = parse("name=").unwrap_err();
    assert_eq!(error.expected, "quoted string");
    assert_eq!(error.found, "");
}
