# Contributing

This workspace uses the pinned nightly from `rust-toolchain.toml`.

Run the baseline checks before opening a PR:

```sh
cargo check --workspace --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo fmt --all --check
cargo doc --workspace --all-features --no-deps
```

Compile-test fixtures live under crate-local `tests/ui/` directories and run through `trybuild`. For macro work, inspect generated code with:

```sh
cargo expand -p tryx-derive
```

The initial toolchain is `nightly-2026-03-28`, selected because it is installed locally and reports `rustc 1.96.0-nightly (fb27476aa 2026-03-28)`. If `try_trait_v2` changes, update the pin and record the reason in this file.
