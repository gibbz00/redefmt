## Non-deferred formatting

Opt-in non-deferred formatting is essential for versatile libraries whom do not
wish to force deferred formatting in non-embedded use-cases.

`redefmt` has made the choice to be entirely no-op if no feature flags are
enabled. No dependencies are pulled in, and nothing exists to be exported. Both
deferred and non-deferred formatting are explicitly opt-in and compatible with
each other.

When the `write-compat` feature flag is enabled, and `deferred` disabled, then `redefmt::Formatter` holds
a `&mut dyn Writer` exactly like `core::fmt`'s `Display` or `Debug` does, and
`redefmt::write` becomes a re-export of `core::write!`.

<!-- TODO: make sure example compiles -->

```rust
let mut stdout_writer = stdout().lock();
let formatter = redefmt::Formatter::new_non_deferred(stdout_writer);
redefmt::Format::fmt(10, formatter).unrwap();
stdout_writer.flush().unwrap();
```

When then `print-compat` is enabled, and `deferred` disabled, then the  `redefmt::print!` becomes
a re-export of `std::print!`.

When then `log-compat` is enabled, and `deferred` disabled, then the  `redefmt::log!` becomes
a re-export of `log::log!`.

`*-compat` is enabled together with `deferred`, all macros become proc macros
expands which expan> the re-exported counterpart, ex. `std::print!`, in addition
to the code for the deferred dispatching.
