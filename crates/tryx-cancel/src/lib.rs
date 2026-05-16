#![feature(try_trait_v2)]
#![no_std]

use core::convert::Infallible;
use core::fmt;

use tryx_core::{ControlFlow, FromResidual, Try, TryxResidual};

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::sync::Arc;

#[cfg(feature = "std")]
use std::sync::atomic::{AtomicBool, Ordering};

/// A computation that can stop early when cancellation is requested.
///
/// # Examples
///
/// ```
/// #![feature(try_trait_v2)]
/// use tryx_cancel::Cancel;
///
/// fn step(cancel: bool) -> Cancel<u8> {
///     if cancel {
///         Cancel::cancelled()
///     } else {
///         Cancel::ok(1)
///     }
/// }
///
/// fn run() -> Cancel<u8> {
///     let value = step(false)?;
///     Cancel::ok(value + 1)
/// }
///
/// assert_eq!(run(), Cancel::Done(2));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cancel<T> {
    /// Completed with a value.
    Done(T),
    /// Stopped because cancellation was requested.
    Cancelled,
}

impl<T> Cancel<T> {
    /// Return a successful cancellation-aware value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::Cancel;
    ///
    /// assert_eq!(Cancel::ok(3), Cancel::Done(3));
    /// ```
    pub fn ok(value: T) -> Self {
        Self::Done(value)
    }

    /// Return a cancelled value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::Cancel;
    ///
    /// assert_eq!(Cancel::<u8>::cancelled(), Cancel::Cancelled);
    /// ```
    pub fn cancelled() -> Self {
        Self::Cancelled
    }

    /// Transform a successful value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::Cancel;
    ///
    /// assert_eq!(Cancel::ok(2).map(|value| value + 1), Cancel::Done(3));
    /// assert_eq!(Cancel::<u8>::cancelled().map(|value| value + 1), Cancel::Cancelled);
    /// ```
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Cancel<U> {
        match self {
            Self::Done(value) => Cancel::Done(f(value)),
            Self::Cancelled => Cancel::Cancelled,
        }
    }

    /// Chain another cancellation-aware computation after a successful value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::Cancel;
    ///
    /// assert_eq!(Cancel::ok(2).and_then(|value| Cancel::ok(value + 1)), Cancel::Done(3));
    /// assert_eq!(
    ///     Cancel::<u8>::cancelled().and_then(|value| Cancel::ok(value + 1)),
    ///     Cancel::Cancelled,
    /// );
    /// ```
    pub fn and_then<U>(self, f: impl FnOnce(T) -> Cancel<U>) -> Cancel<U> {
        match self {
            Self::Done(value) => f(value),
            Self::Cancelled => Cancel::Cancelled,
        }
    }
}

/// Residual produced when a `Cancel<T>` short-circuits.
///
/// # Examples
///
/// ```
/// use tryx_cancel::Cancellation;
///
/// let residual = Cancellation;
/// assert_eq!(residual, Cancellation);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancellation;

impl TryxResidual for Cancellation {}

/// Error adapter used when a cancelled computation is absorbed by `Result`.
///
/// # Examples
///
/// ```
/// #![feature(try_trait_v2)]
/// use tryx_cancel::{Cancel, CancelError};
///
/// #[derive(Debug, PartialEq, Eq)]
/// struct AppError;
///
/// impl From<CancelError> for AppError {
///     fn from(_: CancelError) -> Self {
///         Self
///     }
/// }
///
/// fn run() -> Result<(), AppError> {
///     Cancel::<()>::cancelled()?;
///     Ok(())
/// }
///
/// assert_eq!(run(), Err(AppError));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelError;

impl fmt::Display for CancelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("operation cancelled")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CancelError {}

#[cfg(feature = "std")]
/// Handle checked by cancellation-aware computations.
///
/// # Examples
///
/// ```
/// use tryx_cancel::{Cancel, CancelToken};
///
/// let (token, handle) = CancelToken::new();
/// assert_eq!(token.check(), Cancel::Done(()));
///
/// handle.cancel();
/// assert_eq!(token.check(), Cancel::Cancelled);
/// ```
#[derive(Debug, Clone)]
pub struct CancelToken {
    cancelled: Arc<AtomicBool>,
}

#[cfg(feature = "std")]
/// Handle used to request cancellation from outside a computation.
///
/// # Examples
///
/// ```
/// use tryx_cancel::{Cancel, CancelToken};
///
/// let (token, handle) = CancelToken::new();
/// handle.cancel();
///
/// assert_eq!(token.check(), Cancel::Cancelled);
/// ```
#[derive(Debug, Clone)]
pub struct CancelHandle {
    cancelled: Arc<AtomicBool>,
}

#[cfg(feature = "std")]
impl CancelToken {
    /// Create a token and its paired cancellation handle.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::{Cancel, CancelToken};
    ///
    /// let (token, handle) = CancelToken::new();
    /// assert_eq!(token.check(), Cancel::Done(()));
    ///
    /// handle.cancel();
    /// assert_eq!(token.check(), Cancel::Cancelled);
    /// ```
    pub fn new() -> (Self, CancelHandle) {
        let cancelled = Arc::new(AtomicBool::new(false));
        (
            Self {
                cancelled: Arc::clone(&cancelled),
            },
            CancelHandle { cancelled },
        )
    }

    /// Return `Cancel::Cancelled` if cancellation has been requested.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::{Cancel, CancelToken};
    ///
    /// let (token, handle) = CancelToken::new();
    /// assert_eq!(token.check(), Cancel::Done(()));
    ///
    /// handle.cancel();
    /// assert_eq!(token.check(), Cancel::Cancelled);
    /// ```
    pub fn check(&self) -> Cancel<()> {
        if self.cancelled.load(Ordering::Relaxed) {
            Cancel::Cancelled
        } else {
            Cancel::Done(())
        }
    }
}

#[cfg(feature = "std")]
impl CancelHandle {
    /// Request cancellation.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::{Cancel, CancelToken};
    ///
    /// let (token, handle) = CancelToken::new();
    /// handle.cancel();
    ///
    /// assert_eq!(token.check(), Cancel::Cancelled);
    /// ```
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

impl<T> Try for Cancel<T> {
    type Output = T;
    type Residual = Cancellation;

    fn from_output(output: Self::Output) -> Self {
        Self::ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Done(value) => ControlFlow::Continue(value),
            Self::Cancelled => ControlFlow::Break(Cancellation),
        }
    }
}

impl<T> FromResidual<Cancellation> for Cancel<T> {
    fn from_residual(_: Cancellation) -> Self {
        Self::Cancelled
    }
}

impl<T> FromResidual<Result<Infallible, CancelError>> for Cancel<T> {
    fn from_residual(_: Result<Infallible, CancelError>) -> Self {
        Self::Cancelled
    }
}

impl<T, E> FromResidual<Cancellation> for Result<T, E>
where
    E: From<CancelError>,
{
    fn from_residual(_: Cancellation) -> Self {
        Err(CancelError.into())
    }
}

#[cfg(feature = "cancel-reason")]
/// A computation that can stop early with a caller-defined reason.
///
/// Use this type when the cancellation cause matters to the caller. Keep
/// using `Cancel<T>` when cancellation itself is the only information needed.
///
/// # Examples
///
/// ```
/// #![feature(try_trait_v2)]
/// use tryx_cancel::CancelWith;
///
/// fn step(cancel: bool) -> CancelWith<u8, &'static str> {
///     if cancel {
///         CancelWith::cancelled("stopped by caller")
///     } else {
///         CancelWith::ok(1)
///     }
/// }
///
/// fn run() -> CancelWith<u8, &'static str> {
///     let value = step(false)?;
///     CancelWith::ok(value + 1)
/// }
///
/// assert_eq!(run(), CancelWith::Done(2));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelWith<T, R> {
    /// Completed with a value.
    Done(T),
    /// Stopped with a reason.
    Cancelled(R),
}

#[cfg(feature = "cancel-reason")]
impl<T, R> CancelWith<T, R> {
    /// Return a successful cancellation-aware value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::CancelWith;
    ///
    /// assert_eq!(CancelWith::<_, &str>::ok(3), CancelWith::Done(3));
    /// ```
    pub fn ok(value: T) -> Self {
        Self::Done(value)
    }

    /// Return a cancelled value with a reason.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::CancelWith;
    ///
    /// assert_eq!(
    ///     CancelWith::<u8, _>::cancelled("stopped"),
    ///     CancelWith::Cancelled("stopped"),
    /// );
    /// ```
    pub fn cancelled(reason: R) -> Self {
        Self::Cancelled(reason)
    }

    /// Transform a successful value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::CancelWith;
    ///
    /// assert_eq!(CancelWith::<_, &str>::ok(2).map(|value| value + 1), CancelWith::Done(3));
    /// ```
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> CancelWith<U, R> {
        match self {
            Self::Done(value) => CancelWith::Done(f(value)),
            Self::Cancelled(reason) => CancelWith::Cancelled(reason),
        }
    }

    /// Chain another reason-carrying cancellation-aware computation.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::CancelWith;
    ///
    /// assert_eq!(
    ///     CancelWith::<_, &str>::ok(2).and_then(|value| CancelWith::ok(value + 1)),
    ///     CancelWith::Done(3),
    /// );
    /// ```
    pub fn and_then<U>(self, f: impl FnOnce(T) -> CancelWith<U, R>) -> CancelWith<U, R> {
        match self {
            Self::Done(value) => f(value),
            Self::Cancelled(reason) => CancelWith::Cancelled(reason),
        }
    }
}

#[cfg(feature = "cancel-reason")]
/// Residual produced when `CancelWith<T, R>` short-circuits.
///
/// # Examples
///
/// ```
/// use tryx_cancel::CancellationReason;
///
/// let residual = CancellationReason("stopped");
/// assert_eq!(residual, CancellationReason("stopped"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancellationReason<R>(pub R);

#[cfg(feature = "cancel-reason")]
impl<R> TryxResidual for CancellationReason<R> {}

#[cfg(feature = "cancel-reason")]
/// Error adapter used when `CancelWith<T, R>` is absorbed by `Result`.
///
/// # Examples
///
/// ```
/// use tryx_cancel::CancelWithError;
///
/// assert_eq!(CancelWithError("stopped").reason(), &"stopped");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelWithError<R>(pub R);

#[cfg(feature = "cancel-reason")]
impl<R> CancelWithError<R> {
    /// Borrow the cancellation reason.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::CancelWithError;
    ///
    /// let error = CancelWithError("stopped");
    /// assert_eq!(error.reason(), &"stopped");
    /// ```
    pub fn reason(&self) -> &R {
        &self.0
    }

    /// Return the cancellation reason.
    ///
    /// # Examples
    ///
    /// ```
    /// use tryx_cancel::CancelWithError;
    ///
    /// assert_eq!(CancelWithError("stopped").into_reason(), "stopped");
    /// ```
    pub fn into_reason(self) -> R {
        self.0
    }
}

#[cfg(feature = "cancel-reason")]
impl<R: fmt::Display> fmt::Display for CancelWithError<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "operation cancelled: {}", self.0)
    }
}

#[cfg(all(feature = "std", feature = "cancel-reason"))]
impl<R> std::error::Error for CancelWithError<R> where R: fmt::Debug + fmt::Display {}

#[cfg(feature = "cancel-reason")]
impl<T, R> Try for CancelWith<T, R> {
    type Output = T;
    type Residual = CancellationReason<R>;

    fn from_output(output: Self::Output) -> Self {
        Self::ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Done(value) => ControlFlow::Continue(value),
            Self::Cancelled(reason) => ControlFlow::Break(CancellationReason(reason)),
        }
    }
}

#[cfg(feature = "cancel-reason")]
impl<T, R> FromResidual<CancellationReason<R>> for CancelWith<T, R> {
    fn from_residual(residual: CancellationReason<R>) -> Self {
        Self::Cancelled(residual.0)
    }
}

#[cfg(feature = "cancel-reason")]
impl<T, R> FromResidual<Result<Infallible, CancelWithError<R>>> for CancelWith<T, R> {
    fn from_residual(residual: Result<Infallible, CancelWithError<R>>) -> Self {
        match residual {
            Err(error) => Self::Cancelled(error.into_reason()),
        }
    }
}

#[cfg(feature = "cancel-reason")]
impl<T, R, E> FromResidual<CancellationReason<R>> for Result<T, E>
where
    E: From<CancelWithError<R>>,
{
    fn from_residual(residual: CancellationReason<R>) -> Self {
        Err(CancelWithError(residual.0).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct AppError;

    impl From<CancelError> for AppError {
        fn from(_: CancelError) -> Self {
            Self
        }
    }

    fn maybe_cancel(cancel: bool, value: u8) -> Cancel<u8> {
        if cancel {
            Cancel::Cancelled
        } else {
            Cancel::ok(value)
        }
    }

    fn cancel_returning_success() -> Cancel<u8> {
        let a = maybe_cancel(false, 2)?;
        let b = maybe_cancel(false, 3)?;
        Cancel::Done(a + b)
    }

    fn cancel_returning_short_circuit() -> Cancel<u8> {
        let _ = maybe_cancel(true, 2)?;
        Cancel::Done(9)
    }

    fn result_returning_short_circuit() -> Result<u8, AppError> {
        let _ = maybe_cancel(true, 2)?;
        Ok(9)
    }

    #[test]
    fn question_mark_continues_inside_cancel() {
        assert_eq!(cancel_returning_success(), Cancel::Done(5));
    }

    #[test]
    fn question_mark_short_circuits_inside_cancel() {
        assert_eq!(cancel_returning_short_circuit(), Cancel::Cancelled);
    }

    #[test]
    fn question_mark_converts_cancel_into_result_error() {
        assert_eq!(result_returning_short_circuit(), Err(AppError));
    }

    #[cfg(feature = "cancel-reason")]
    mod reason {
        use super::*;

        #[derive(Debug, PartialEq, Eq)]
        struct ReasonError(&'static str);

        impl From<CancelWithError<&'static str>> for ReasonError {
            fn from(error: CancelWithError<&'static str>) -> Self {
                Self(error.into_reason())
            }
        }

        fn maybe_cancel(cancel: bool) -> CancelWith<u8, &'static str> {
            if cancel {
                CancelWith::cancelled("stopped")
            } else {
                CancelWith::ok(4)
            }
        }

        fn cancel_with_returning_success() -> CancelWith<u8, &'static str> {
            let value = maybe_cancel(false)?;
            CancelWith::ok(value + 1)
        }

        fn cancel_with_returning_short_circuit() -> CancelWith<u8, &'static str> {
            let _ = maybe_cancel(true)?;
            CancelWith::ok(9)
        }

        fn result_returning_reason() -> Result<u8, ReasonError> {
            let _ = maybe_cancel(true)?;
            Ok(9)
        }

        #[test]
        fn question_mark_continues_inside_cancel_with() {
            assert_eq!(cancel_with_returning_success(), CancelWith::Done(5));
        }

        #[test]
        fn question_mark_short_circuits_inside_cancel_with() {
            assert_eq!(
                cancel_with_returning_short_circuit(),
                CancelWith::Cancelled("stopped"),
            );
        }

        #[test]
        fn question_mark_converts_cancel_with_into_result_error() {
            assert_eq!(result_returning_reason(), Err(ReasonError("stopped")));
        }
    }
}
