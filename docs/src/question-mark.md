# The Question Mark Operator

Rust's `?` operator delegates to `Try` and `FromResidual`. `tryx` uses those nightly traits to model outcomes that are not naturally `Result` or `Option`.

The important rule is simple: the success value continues, and the residual short-circuits.
