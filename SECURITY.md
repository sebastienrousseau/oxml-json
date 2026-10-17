# Security Policy

## Supported versions

oxml-json is pre-1.0. Only the most recent `0.0.x` release receives security
fixes.

## Reporting a vulnerability

Please report privately, not as a public issue:

- GitHub's [private vulnerability reporting](https://github.com/sebastienrousseau/oxml-json/security/advisories/new)
- or email <sebastian.rousseau@gmail.com>

You should get an acknowledgement within 72 hours.

## Threat model

This crate parses JSON, not XML. `parse` rejects sources larger than
16 MiB and more than 256 nested arrays/objects. It owns its parsed strings
and builds the complete value, so it is not a constant-memory streaming
parser. Applications must also bound transport buffering and concurrency.

Numbers use finite `f64`; integers beyond 2^53 can lose precision. Do not
use this representation for identifiers requiring larger exact integers.
Duplicate object keys retain the last value. Callers must not combine this
with a verifier that interprets duplicate keys differently. `Json` values
constructed directly by a caller are trusted data: the infallible writer
does not validate arbitrary nesting or non-finite manually built numbers.

**Memory safety.** This crate forbids unsafe Rust in its own source.
Dependencies and runtimes have separate safety obligations. This
does not rule out resource exhaustion, logic defects or protocol errors.

## What is not a vulnerability

- Rejecting JSON another parser accepts, when that input violates
  the JSON grammar. Please still report it — it is a bug, just not a
  security one.
