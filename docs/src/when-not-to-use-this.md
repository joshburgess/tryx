# When Not To Use This

Do not use a custom outcome type when `Result` would do. `?` already works on `Result`, and most public Rust APIs should expose ordinary error types.

Do not use outcome types whose short-circuit case is common. `?` hides the branch, so frequent control flow deserves an explicit `match`.

Do not put these types in public APIs of widely used crates unless the semantics are central to the crate. Callers should not have to learn a new meaning for `?` just to use a basic function.

Do not compose outcome types when residual semantics conflict. If a function wants to return both parsing failure and cancellation, pick one outcome for the function body and convert at the boundary.

Do not use `Stage` when a plain state machine is clearer. `Stage` is useful when partial state and resumption are part of the contract, not when the code only has several helper functions.

Do not use `Checked<Finite>` as a general numeric tower. It is a narrow tool for making invalid floating-point results visible in `?`-driven code.
