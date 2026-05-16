# tryx

`tryx` is an experimental nightly Rust library for outcome types that work with the `?` operator beyond `Result` and `Option`.

It currently includes:

- `Cancel<T>` for cancellation-aware control flow
- `Checked<Finite>` for checked floating-point operations
- `Parsing<'a, T>` for parser-style outcomes with diagnostics
- `Stage<S, T, P>` for staged computations that retain partial state
- `#[derive(Outcome)]` for simple custom outcome enums

`tryx` requires nightly Rust and `try_trait_v2`. The workspace pins `nightly-2026-03-28`.

## Example

```rust
#![feature(try_trait_v2)]

use tryx::cancel::{Cancel, CancelToken};

fn work(token: &CancelToken) -> Cancel<u32> {
    token.check()?;
    Cancel::ok(42)
}

let (token, handle) = CancelToken::new();
assert_eq!(work(&token), Cancel::Done(42));

handle.cancel();
assert_eq!(work(&token), Cancel::Cancelled);
```

## Docs

Build the book and API docs locally:

```sh
mdbook build docs
cargo doc --workspace --all-features --no-deps
```

Run the examples:

```sh
cargo run -p tryx --example cancel_basic --features cancel
cargo run -p tryx --example quadratic --features checked
cargo run -p tryx --example kv_parser --features parsing
cargo run -p tryx --example etl_pipeline --features stage
```
