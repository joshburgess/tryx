# Changelog

## Unreleased

- Scaffold the workspace, toolchain, CI, and documentation shell.

## v0.1.0

- Add `Cancel<T>` and `CancelWith<T, R>`.
- Add `Checked<Finite>` and checked float arithmetic.
- Add `Parsing<'a, T>` with owned `ParseError` interop.
- Add `Stage<S, T, P>` for resumable staged computations.
- Add `#[derive(Outcome)]` for simple custom outcome enums.
