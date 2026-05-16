use tryx::parsing::{ParseError, eof, ident, literal, quoted_string};

fn parse_line(input: &str) -> Result<(&str, &str), ParseError> {
    let (key, rest) = ident(input)?;
    let (_, rest) = literal(rest, '=')?;
    let (value, rest) = quoted_string(rest)?;
    let (_, _) = eof(rest)?;
    Ok((key, value))
}

fn main() -> Result<(), ParseError> {
    for line in ["name=\"tryx\"", "kind=\"library\""] {
        let (key, value) = parse_line(line)?;
        println!("{key}={value}");
    }

    Ok(())
}
