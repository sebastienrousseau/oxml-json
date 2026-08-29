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

**1. Borrowed parsing.** Every string in a parsed document is owned
today, which means a copy per string. Ranges into the input would
remove that, at the cost of a lifetime on `Json`. Worth measuring
before committing to the API change: the documents this handles are
MCP request bodies, and for those the copy may not be the cost that
matters.

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
