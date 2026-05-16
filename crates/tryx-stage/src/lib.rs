#![feature(try_trait_v2)]
#![no_std]

use core::convert::Infallible;
use core::fmt;

use tryx_core::{ControlFlow, FromResidual, Try, TryxResidual};

#[cfg(feature = "std")]
extern crate std;

/// Outcome for a staged computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage<S, T, P = ()> {
    Done(T),
    Failed(StageFailure<S, P>),
}

impl<S, T, P> Stage<S, T, P> {
    pub fn done(value: T) -> Self {
        Self::Done(value)
    }

    pub fn failed(stage: S, partial: P) -> Self {
        Self::Failed(StageFailure { stage, partial })
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Stage<S, U, P> {
        match self {
            Self::Done(value) => Stage::Done(f(value)),
            Self::Failed(failure) => Stage::Failed(failure),
        }
    }
}

/// Failure from a staged computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageFailure<S, P = ()> {
    pub stage: S,
    pub partial: P,
}

impl<S, P> TryxResidual for StageFailure<S, P> {}

/// Error adapter used when a staged failure is absorbed by `Result`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageError<S> {
    pub stage: S,
}

impl<S: fmt::Debug> fmt::Display for StageError<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pipeline stopped at stage {:?}", self.stage)
    }
}

#[cfg(feature = "std")]
impl<S: fmt::Debug> std::error::Error for StageError<S> {}

impl<S, T, P> Try for Stage<S, T, P> {
    type Output = T;
    type Residual = StageFailure<S, P>;

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

impl<S, T, P> FromResidual<StageFailure<S, P>> for Stage<S, T, P> {
    fn from_residual(residual: StageFailure<S, P>) -> Self {
        Self::Failed(residual)
    }
}

impl<S, T, P> FromResidual<Result<Infallible, StageError<S>>> for Stage<S, T, P>
where
    P: Default,
{
    fn from_residual(residual: Result<Infallible, StageError<S>>) -> Self {
        match residual {
            Err(error) => Self::Failed(StageFailure {
                stage: error.stage,
                partial: P::default(),
            }),
        }
    }
}

impl<S, T, P, E> FromResidual<StageFailure<S, P>> for Result<T, E>
where
    E: From<StageError<S>>,
{
    fn from_residual(residual: StageFailure<S, P>) -> Self {
        Err(StageError {
            stage: residual.stage,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Step {
        Parse,
    }

    fn parse(input: &str) -> Stage<Step, u32, &'_ str> {
        match input.parse::<u32>() {
            Ok(value) => Stage::done(value),
            Err(_) => Stage::failed(Step::Parse, input),
        }
    }

    #[test]
    fn question_mark_short_circuits_with_partial_state() {
        fn run() -> Stage<Step, u32, &'static str> {
            let value = parse("bad")?;
            Stage::done(value + 1)
        }

        assert_eq!(
            run(),
            Stage::Failed(StageFailure {
                stage: Step::Parse,
                partial: "bad"
            })
        );
    }
}
