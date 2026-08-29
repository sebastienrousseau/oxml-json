<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Assurance case

An assurance case is an argument, supported by evidence, that the
software is adequately secure for what it does. This one is
deliberately short: the strongest security claim this project makes is
about what it *cannot* do.

## What this software is

`oxml-json` is a JSON value, parser and serialiser.

## What it consumes

Its inputs are JSON-RPC messages arriving over a pipe from an editor
or a language model. The threat model assumes every one of them is
hostile: a document written specifically to crash the parser, exhaust
memory, or reach something it should not.

## The claim

**A hostile input can cause this software to return an error. It
cannot cause it to corrupt memory, execute code, exhaust the machine,
or reach the network or the filesystem.**

## The argument

### Memory safety is structural, not tested for

This crate opens no file and no socket. It turns bytes into a value and back; there is nothing else for it to reach.

### Resource exhaustion is bounded, not merely unlikely

Depth, entity expansion and input size are bounded by explicit limits
with documented defaults. Recursion is bounded because a stack
overflow aborts the process rather than unwinding, and no caller can
catch it.

### Correctness is measured against an external standard

The project does not grade its own homework. Where an independent
conformance suite exists it is run, its denominator is published
alongside its rate, and the result is ratcheted so an unreviewed change
in either direction fails the build.

## The evidence

- `#![forbid(unsafe_code)]`, checked by a CI job that greps for the
  attribute rather than trusting it is still there.
- 19 tests and a doctest; **100% line and function coverage**, with a
  95% floor gated in CI and branch coverage gated at 80.
- A `cargo-fuzz` target on every pull request. It asserts more than the
  absence of a panic: anything that parses must survive a round trip
  through `to_json` and back to the same value.
- That assertion is why this crate exists in the form it does. It
  found, within ninety seconds of the extraction, a defect that both
  copies of this parser had carried for months: `1e999` parsed to
  infinity and serialised to the bare token `inf`, which is not JSON.
- Numbers outside `f64`'s range are refused rather than rounded, so a
  value this crate accepts is a value it can write back.

## What this case does *not* claim

- It does not claim the absence of defects. It claims that a defect of
  a particular class — memory corruption — is ruled out by
  construction, and that other classes are bounded and tested for.
- It does not claim the defaults are the tightest possible. They are
  chosen to accept every real document encountered; a service parsing
  untrusted XML under load should tighten them.
- It does not claim independent review. This project has one
  maintainer, and no third party has audited it. That is recorded here
  rather than left to be inferred.

## Reporting a problem with this case

If you can construct an input that violates the claim above, that is a
vulnerability. See [SECURITY.md](../SECURITY.md).
