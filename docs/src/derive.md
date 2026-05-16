# Defining Outcomes

`#[derive(Outcome)]` supports enums with one success variant and one short-circuit variant.

```rust
#![feature(try_trait_v2)]
use tryx_derive::Outcome;

#[derive(Outcome)]
#[outcome(result_interop = AppError)]
enum Permitted<T> {
    #[outcome(success)]
    Yes(T),
    #[outcome(short_circuit)]
    No(AppError),
}

#[derive(Debug)]
struct AppError;
```

The derive intentionally rejects ambiguous shapes, including multiple success variants and short-circuit variants with multiple fields.
