#![cfg(feature = "checked")]

use tryx::checked::{Checked, Finite, FloatFailure};

fn solve_sum(a: f64, b: f64) -> Checked<Finite> {
    let a = Finite::new(a)?;
    let b = Finite::new(b)?;
    a + b
}

#[test]
fn umbrella_reexports_checked_api() {
    assert_eq!(solve_sum(2.0, 3.0).into_result().map(Finite::get), Ok(5.0));
    assert_eq!(solve_sum(f64::NAN, 3.0), Checked::Failed(FloatFailure::Nan));
}

#[test]
fn conversion_paths_work() {
    let value = Finite::try_from(7.0);
    assert_eq!(value.map(Finite::get), Ok(7.0));
    assert_eq!(
        Finite::try_from(f64::NEG_INFINITY),
        Err(FloatFailure::NegInfinity)
    );

    let checked = Finite::new(4.0);
    assert_eq!(checked.into_result().map(f64::from), Ok(4.0));
}
