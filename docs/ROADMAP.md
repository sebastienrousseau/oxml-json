<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Roadmap

## Where this is

A JSON parser and serialiser with no dependencies, extracted from
`oxml-mcp` at 0.0.8 so that the MCP server and the command-line tool
share one implementation rather than two that could disagree.

The surface is deliberately small: `parse` a string into a `Json`, and
`to_json` a `Json` back into a string. It is not a general-purpose
serialisation framework and is not trying to become one — `serde_json`
exists and is better at that. What this gives up in features it buys
back in having nothing to audit.

## The order

**1. Borrowed parsing — measured, and it is not the win it looked
like.** Every string in a parsed document is owned, which means a copy
per string. Ranges into the input would remove that, at the cost of a
lifetime on `Json`.

`benches/parse.rs` compares two documents of the same entry count that
differ in how much of each entry is string content, normalised per
byte. Owning the strings does not dominate: the string-heavy shape
parses at **0.75-0.89x the per-byte cost** of the numeric one across
three runs — that is, *faster* per byte, because parsing a number
costs more than copying a short string. The spread is machine noise;
the direction was the same every time.

So the lifetime is not worth adding for the bodies this crate sees.
An MCP request parses in about 1.5 us and a `tools/list` reply in
about 9 us; the copy is not where that time goes. Revisit only if a
caller appears whose documents are both large and overwhelmingly
textual.

**2. A streaming reader.** `parse` builds the whole value. A
response large enough to matter is one that should not be held whole,
which is the same argument `oxml`'s `stream` module makes for XML.

**3. Number fidelity.** Numbers are `f64`. An integer beyond 2^53
round-trips to a different value, and JSON does not say it must not.
Callers that need exactness need something else here, and the type
should say so before it is relied on.

## What is deliberately absent

**No `serde` integration.** It would pull in the dependency this crate
exists to avoid. A caller who wants `serde` should use `serde_json`.

**No trailing commas, comments or `NaN`.** These are extensions, not
JSON, and accepting them silently makes the parser disagree with every
other one.

## Non-goals

Performance parity with `simd-json`. This is a correct, small,
dependency-free parser; a vectorised one is a different project with
different trade-offs.
