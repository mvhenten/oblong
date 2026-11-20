# Agent rules

Before every push:

1. Run `rustup update stable`.
2. Run `cargo fmt -- --check` and `cargo clippy -- -D warnings`, the same commands as CI.
3. Fix any finding at its source. Never add `#[allow]`.
