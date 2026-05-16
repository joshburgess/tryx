#![feature(try_trait_v2)]

use tryx_derive::Outcome;

#[derive(Debug, PartialEq, Eq)]
struct AppError(&'static str);

impl From<&'static str> for AppError {
    fn from(value: &'static str) -> Self {
        Self(value)
    }
}

#[derive(Outcome)]
#[outcome(result_interop = AppError)]
enum Permitted<T> {
    #[outcome(success)]
    Yes(T),
    #[outcome(short_circuit)]
    No(&'static str),
}

fn permitted() -> Permitted<u8> {
    let value = Permitted::Yes(2)?;
    Permitted::Yes(value + 1)
}

fn result() -> Result<u8, AppError> {
    let value = Permitted::Yes(3)?;
    Ok(value)
}

fn main() {
    assert!(matches!(permitted(), Permitted::Yes(3)));
    assert_eq!(result(), Ok(3));
}
