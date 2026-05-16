# 0001: Cancellation Reasons

## Status

Accepted

## Decision

Reason-carrying cancellation uses a sibling type, `CancelWith<T, R>`, behind the `cancel-reason` feature.

The base `Cancel<T>` type remains reasonless. It represents the common case where callers only need to stop work, not explain why work stopped.

## Rationale

Adding a reason parameter to `Cancel<T>` would make the simplest API pay for a feature many callers do not need. It would also force basic examples to pick a reason type before the reason has any value.

The sibling type keeps both use cases explicit:

- `Cancel<T>`: stop or continue
- `CancelWith<T, R>`: stop with a caller-defined reason

## Consequences

`Cancel<T>` and `CancelWith<T, R>` are separate outcome types. Callers convert between them at API boundaries instead of relying on one type to cover both semantics.

The project will not ship both a reason-parameterized `Cancel<T, R>` and `CancelWith<T, R>`.
