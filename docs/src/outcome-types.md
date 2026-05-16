# Outcome Types

## Cancel

`Cancel<T>` stops work when cancellation is requested. Use `CancelToken` when another thread or owner needs to request cancellation.

Run:

```sh
cargo run -p tryx --example cancel_basic --features cancel
cargo run -p tryx --example cancel_render_loop --features cancel
```

## Checked

`Finite` wraps a finite `f64`. `Checked<Finite>` is the `?`-able outcome used by constructors and arithmetic.

Run:

```sh
cargo run -p tryx --example quadratic --features checked
```

## Parsing

`Parsing<'a, T>` carries parsed values, remaining input, and borrowed failures. `ParseError` owns the diagnostic data when a parser is absorbed by `Result`.

Run:

```sh
cargo run -p tryx --example kv_parser --features parsing
```

## Stage

`Stage<S, T, P>` carries the stage marker and partial state when a staged computation stops.

Run:

```sh
cargo run -p tryx --example etl_pipeline --features stage
```
