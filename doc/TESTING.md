<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Testing

## What is checked

19 tests and a doctest over `parse` and `to_json`: every value type,
nesting, escapes, surrogate pairs, the numeric edge cases, and round
trips.

Line and function coverage are both **100%**; region coverage is
99.36%. Line coverage is gated in CI at a 95% floor and branch
coverage at 80%, so the figures above are a result rather than a
target -- the gates are where they are to catch erosion, not to
describe the crate.

## The cases worth naming

**Numbers that do not fit an `f64`.** `1e999` parses to infinity in
Rust without error, and infinity formats as the bare token `inf`,
which is not JSON. This crate rejects such a literal instead. The bug
was real and shipped: `oxml-mcp` 0.0.7 echoed a request id of `1e999`
back as `{"id":inf,...}`, and a client parsing that reply failed.

**Surrogate pairs.** `😀` is one character written as two
escapes. Decoding them independently produces two unpaired surrogates
and a string that is not valid UTF-8.

**Round trips.** `parse(value.to_json()) == value` is asserted by the
tests and by the fuzz target, so the two directions are checked
against each other rather than each against a fixture that could be
wrong in the same way.

## Fuzzing

One target, `parse`, run for 300 seconds on every pull request:

```bash
cargo +nightly fuzz run parse
```

It asserts the round trip rather than merely the absence of a panic.
A parser that accepts input and then serialises it into something it
cannot read back is broken even though nothing crashed, and that is
the failure a panic-only target misses.

## Running everything

```bash
cargo test --all-features        # 19 tests and a doctest
./scripts/gate.sh                # everything CI runs, in the same order
```

`gate.sh` is the same set of checks the CI workflow runs, so a green
gate locally means a green CI -- provided the toolchain matches.
`rust-toolchain.toml` pins it; a `RUSTUP_TOOLCHAIN` set in the
environment silently overrides that file, which is how a local run can
disagree with CI while appearing identical.
