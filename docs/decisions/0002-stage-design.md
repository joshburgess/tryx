# 0002: Stage Design

## Status

Accepted

## Decision

`Stage<S, T, P = ()>` represents a staged computation.

- `S` is the caller's stage marker type.
- `T` is the success value.
- `P` is partial state carried when the computation stops.

The failure case is `StageFailure<S, P>`. Resumption is explicit: callers inspect the failure, recover or update the partial state, then call the next stage function themselves.

## Rationale

Phantom marker types and caller-owned enums are more readable than const generic stage numbers for public APIs. They let examples name domain stages directly, such as `Load`, `Transform`, and `Write`.

Automatic resumption would require a pipeline builder or macro that knows every stage transition. That is useful later, but it is too much machinery for v0.1.

## Result Interop

`Result` interop converts `StageFailure<S, P>` into `StageError<S>`, dropping the partial state. Callers that need resumability should return `Stage` directly instead of `Result`.
