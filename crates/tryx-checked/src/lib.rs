#![feature(try_trait_v2)]
#![no_std]

use core::convert::Infallible;
use core::fmt;
use core::ops::{Add, Div, FromResidual, Mul, Rem, Sub, Try};

use tryx_core::{ControlFlow, TryxResidual};

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A finite `f64` value.
///
/// # Examples
///
/// ```
/// use tryx_checked::{Checked, Finite};
///
/// assert!(matches!(Finite::new(1.0), Checked::Done(_)));
/// assert!(matches!(Finite::new(f64::NAN), Checked::Failed(_)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Finite(f64);

impl Finite {
    /// Check that `value` is finite.
    ///
    /// Returns `FloatFailure::Nan`, `FloatFailure::PosInfinity`, or
    /// `FloatFailure::NegInfinity` for non-finite inputs.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_checked::{Checked, Finite, FloatFailure};
    ///
    /// assert_eq!(Finite::new(f64::INFINITY), Checked::Failed(FloatFailure::PosInfinity));
    /// ```
    pub fn new(value: f64) -> Checked<Self> {
        classify(value).map(Self)
    }

    /// Return the wrapped `f64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_checked::Finite;
    ///
    /// let value = Finite::try_from(2.0)?;
    /// assert_eq!(value.get(), 2.0);
    /// # Ok::<(), tryx_checked::FloatFailure>(())
    /// ```
    pub fn get(self) -> f64 {
        self.0
    }

    /// Return the square root.
    ///
    /// Negative inputs produce `FloatFailure::Nan`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_checked::{Checked, Finite, FloatFailure};
    ///
    /// let value = Finite::try_from(9.0)?;
    /// assert_eq!(value.sqrt(), Checked::Done(Finite::try_from(3.0)?));
    /// assert_eq!(Finite::try_from(-1.0)?.sqrt(), Checked::Failed(FloatFailure::Nan));
    /// # Ok::<(), tryx_checked::FloatFailure>(())
    /// ```
    #[cfg(feature = "std")]
    pub fn sqrt(self) -> Checked<Self> {
        Self::new(self.0.sqrt())
    }

    /// Return the natural logarithm.
    ///
    /// Negative inputs produce `FloatFailure::Nan`; zero produces
    /// `FloatFailure::NegInfinity`.
    #[cfg(feature = "std")]
    pub fn ln(self) -> Checked<Self> {
        Self::new(self.0.ln())
    }

    /// Return the logarithm in `base`.
    ///
    /// Invalid bases or negative inputs produce `FloatFailure::Nan`; zero input
    /// can produce `FloatFailure::NegInfinity`.
    #[cfg(feature = "std")]
    pub fn log(self, base: Self) -> Checked<Self> {
        Self::new(self.0.log(base.0))
    }

    /// Return the base-2 logarithm.
    ///
    /// Negative inputs produce `FloatFailure::Nan`; zero produces
    /// `FloatFailure::NegInfinity`.
    #[cfg(feature = "std")]
    pub fn log2(self) -> Checked<Self> {
        Self::new(self.0.log2())
    }

    /// Return the base-10 logarithm.
    ///
    /// Negative inputs produce `FloatFailure::Nan`; zero produces
    /// `FloatFailure::NegInfinity`.
    #[cfg(feature = "std")]
    pub fn log10(self) -> Checked<Self> {
        Self::new(self.0.log10())
    }

    /// Return `e.powf(self)`.
    ///
    /// Overflow produces an infinity failure.
    #[cfg(feature = "std")]
    pub fn exp(self) -> Checked<Self> {
        Self::new(self.0.exp())
    }

    /// Raise this value to `power`.
    ///
    /// Invalid fractional powers of negative values produce `FloatFailure::Nan`;
    /// overflow produces an infinity failure.
    #[cfg(feature = "std")]
    pub fn pow(self, power: Self) -> Checked<Self> {
        Self::new(self.0.powf(power.0))
    }

    /// Return the sine.
    ///
    /// Finite inputs are expected to produce finite outputs.
    #[cfg(feature = "std")]
    pub fn sin(self) -> Checked<Self> {
        Self::new(self.0.sin())
    }

    /// Return the cosine.
    ///
    /// Finite inputs are expected to produce finite outputs.
    #[cfg(feature = "std")]
    pub fn cos(self) -> Checked<Self> {
        Self::new(self.0.cos())
    }

    /// Return the tangent.
    ///
    /// Very large finite inputs may produce an invalid result on some platforms.
    #[cfg(feature = "std")]
    pub fn tan(self) -> Checked<Self> {
        Self::new(self.0.tan())
    }
}

impl TryFrom<f64> for Finite {
    type Error = FloatFailure;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value).into_result()
    }
}

impl From<Finite> for f64 {
    fn from(value: Finite) -> Self {
        value.0
    }
}

#[cfg(feature = "serde")]
impl Serialize for Finite {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Finite {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

/// Outcome for checked floating-point operations.
///
/// # Examples
///
/// ```
/// #![feature(try_trait_v2)]
/// use tryx_checked::{Checked, Finite};
///
/// fn run() -> Checked<Finite> {
///     let value = Finite::new(2.0)?;
///     value.sqrt()
/// }
///
/// assert!(matches!(run(), Checked::Done(_)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Checked<T> {
    /// Completed with a checked value.
    Done(T),
    /// Stopped because a float operation produced an invalid value.
    Failed(FloatFailure),
}

impl<T> Checked<T> {
    /// Transform a successful value.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Checked<U> {
        match self {
            Self::Done(value) => Checked::Done(f(value)),
            Self::Failed(failure) => Checked::Failed(failure),
        }
    }
}

impl Checked<Finite> {
    /// Convert into a standard `Result`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_checked::{Finite, FloatFailure};
    ///
    /// assert_eq!(Finite::new(f64::NAN).into_result(), Err(FloatFailure::Nan));
    /// ```
    pub fn into_result(self) -> Result<Finite, FloatFailure> {
        match self {
            Self::Done(value) => Ok(value),
            Self::Failed(failure) => Err(failure),
        }
    }
}

impl<T> Try for Checked<T> {
    type Output = T;
    type Residual = FloatFailure;

    fn from_output(output: Self::Output) -> Self {
        Self::Done(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Done(value) => ControlFlow::Continue(value),
            Self::Failed(failure) => ControlFlow::Break(failure),
        }
    }
}

impl<T> FromResidual<FloatFailure> for Checked<T> {
    fn from_residual(residual: FloatFailure) -> Self {
        Self::Failed(residual)
    }
}

impl<T> FromResidual<Result<Infallible, FloatFailure>> for Checked<T> {
    fn from_residual(residual: Result<Infallible, FloatFailure>) -> Self {
        match residual {
            Err(failure) => Self::Failed(failure),
        }
    }
}

impl<T, E> FromResidual<FloatFailure> for Result<T, E>
where
    E: From<FloatFailure>,
{
    fn from_residual(residual: FloatFailure) -> Self {
        Err(residual.into())
    }
}

/// Failure produced by checked floating-point construction or arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatFailure {
    Nan,
    PosInfinity,
    NegInfinity,
    #[cfg(feature = "subnormal")]
    Subnormal,
}

impl TryxResidual for FloatFailure {}

impl fmt::Display for FloatFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nan => f.write_str("float operation produced NaN"),
            Self::PosInfinity => f.write_str("float operation produced positive infinity"),
            Self::NegInfinity => f.write_str("float operation produced negative infinity"),
            #[cfg(feature = "subnormal")]
            Self::Subnormal => f.write_str("float operation produced a subnormal value"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FloatFailure {}

impl Add for Finite {
    type Output = Checked<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}

impl Add<f64> for Finite {
    type Output = Checked<Self>;

    fn add(self, rhs: f64) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}

impl Sub for Finite {
    type Output = Checked<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0 - rhs.0)
    }
}

impl Sub<f64> for Finite {
    type Output = Checked<Self>;

    fn sub(self, rhs: f64) -> Self::Output {
        Self::new(self.0 - rhs)
    }
}

impl Mul for Finite {
    type Output = Checked<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.0 * rhs.0)
    }
}

impl Mul<f64> for Finite {
    type Output = Checked<Self>;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.0 * rhs)
    }
}

impl Div for Finite {
    type Output = Checked<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.0 / rhs.0)
    }
}

impl Div<f64> for Finite {
    type Output = Checked<Self>;

    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.0 / rhs)
    }
}

impl Rem for Finite {
    type Output = Checked<Self>;

    fn rem(self, rhs: Self) -> Self::Output {
        Self::new(self.0 % rhs.0)
    }
}

impl Rem<f64> for Finite {
    type Output = Checked<Self>;

    fn rem(self, rhs: f64) -> Self::Output {
        Self::new(self.0 % rhs)
    }
}

fn classify(value: f64) -> Checked<f64> {
    if value.is_nan() {
        Checked::Failed(FloatFailure::Nan)
    } else if value == f64::INFINITY {
        Checked::Failed(FloatFailure::PosInfinity)
    } else if value == f64::NEG_INFINITY {
        Checked::Failed(FloatFailure::NegInfinity)
    } else {
        #[cfg(feature = "subnormal")]
        if value != 0.0 && value.is_subnormal() {
            return Checked::Failed(FloatFailure::Subnormal);
        }

        Checked::Done(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn f(value: f64) -> Finite {
        match Finite::try_from(value) {
            Ok(value) => value,
            Err(failure) => panic!("test value should be finite: {failure}"),
        }
    }

    proptest! {
        #[test]
        fn new_accepts_exactly_finite_values(value in any::<f64>()) {
            let expected = value.is_finite()
                && {
                    #[cfg(feature = "subnormal")]
                    {
                        value == 0.0 || !value.is_subnormal()
                    }
                    #[cfg(not(feature = "subnormal"))]
                    {
                        true
                    }
                };
            prop_assert_eq!(matches!(Finite::new(value), Checked::Done(_)), expected);
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn question_mark_short_circuits_nan() {
        fn run() -> Checked<Finite> {
            let _ = Finite::new(0.0 / 0.0)?;
            Finite::new(1.0)
        }

        assert_eq!(run(), Checked::Failed(FloatFailure::Nan));
    }

    #[test]
    fn result_interop_converts_failure() {
        fn run() -> Result<Finite, FloatFailure> {
            let value = Finite::new(f64::INFINITY)?;
            Ok(value)
        }

        assert_eq!(run(), Err(FloatFailure::PosInfinity));
    }

    #[cfg(feature = "std")]
    #[test]
    fn arithmetic_checks_results() {
        assert_eq!((f(4.0) + f(5.0)).into_result(), Ok(f(9.0)));
        assert_eq!(f(1.0) / 0.0, Checked::Failed(FloatFailure::PosInfinity));
        assert_eq!(f(-1.0).sqrt(), Checked::Failed(FloatFailure::Nan));
    }
}
