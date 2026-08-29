<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<h1 align="center">oxml-json</h1>

<p align="center">
  A small JSON value, parser and serialiser for the oxml suite's
  JSON-RPC crates.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/oxml-json/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/oxml-json/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/oxml-json"><img src="https://img.shields.io/crates/v/oxml-json.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/oxml-json"><img src="https://img.shields.io/badge/docs.rs-oxml--json-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
</p>

---

## Why this exists

Two crates in the [oxml](https://github.com/sebastienrousseau/oxml)
suite speak JSON-RPC: `oxml-mcp` over the Model Context Protocol, and
`oxml-lsp` over the Language Server Protocol. For a while each carried
its own copy of this file, because neither could reach the other's
private module.

A copy is fine until it is not. A defect fixed in one would not have
reached the other, and nothing would have noticed — the failure mode
this suite keeps finding elsewhere. So it lives here instead.

**The extraction was worth it immediately.** Moving the fuzz target
here came with a stronger assertion — that anything which parses must
survive a round trip — and it found a defect within ninety seconds that
had been in both copies: `1e999` is grammatically valid JSON, parses to
infinity, and infinity has no JSON representation, so serialising it
produced output that would not parse. It is now refused, because
writing `null` instead would silently change the value.

## Scope

JSON-RPC messages are small and their shapes are fixed, so this
implements what those protocols need rather than everything the grammar
allows. It is not a general-purpose JSON library and does not try to
be. **If you want `serde`, use `serde`** — it is better at this, and
this crate exists to avoid a dependency in a specific place, not to
compete.

## Install

```toml
[dependencies]
oxml-json = "0.0.8"
```

## Quick Start

```rust
use oxml_json::{parse, Json};

let value = parse(r#"{"jsonrpc":"2.0","id":1}"#).expect("valid JSON");
assert_eq!(value.get("jsonrpc").and_then(Json::as_str), Some("2.0"));

let reply = Json::object(vec![
    ("jsonrpc", Json::str("2.0")),
    ("id", Json::Number(1.0)),
]);
assert_eq!(reply.to_json(), r#"{"id":1,"jsonrpc":"2.0"}"#);
```

Object keys come back out in sorted order, which makes serialised
output comparable in a test without parsing it again.

## Numbers

Numbers are `f64`, which is what JSON's grammar describes and what both
protocols use for ids and codes. An integer beyond 2<sup>53</sup> does
not survive that, which no request id in either protocol reaches. A
literal outside `f64`'s range is **refused** rather than rounded to
infinity.

## Development

```bash
./scripts/gate.sh
```

Everything CI runs: format, clippy, tests, rustdoc, the
`#![forbid(unsafe_code)]` check, the example, a 95% coverage floor and
an MSRV build. Fuzzing runs in CI on every pull request.

## Security

See [SECURITY.md](SECURITY.md). This crate parses untrusted input by
design — both protocols carry it — so it is fuzzed continuously and
refuses malformed input rather than guessing at it.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
