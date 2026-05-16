# Result Interop

Every shipped residual can be absorbed by a compatible `Result`.

Use this at boundaries where callers expect normal Rust error handling. Keep the custom outcome internally when its short-circuit semantics matter to the implementation.
