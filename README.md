# tryx

`tryx` is an experimental Rust workspace for outcome types that work with the `?` operator beyond `Result` and `Option`.

The library is nightly-only and requires `try_trait_v2`. The workspace pins `nightly-2026-03-28`, the locally available known-good toolchain used to start the project.

```rust
#![feature(try_trait_v2)]

// API examples will land with the first implemented outcome type.
```

The build plan lives in [PROJECT_BUILD_PLAN.md](PROJECT_BUILD_PLAN.md). The mdBook source will live under `docs/`.
