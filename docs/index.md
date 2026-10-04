---
title: oxml-json — Lightweight JSON Parser & Serializer
description: A small JSON value, parser, and serializer for the oxml suite's JSON-RPC crates.
hide:
  - navigation
  - toc
---

<section class="dot-hero" markdown>

# oxml-json

<p class="tagline">A small, zero-unsafe JSON value, parser, and serializer for the oxml suite's JSON-RPC, LSP, and MCP crates.</p>

<div class="buttons">
  <a class="primary" href="https://docs.rs/oxml-json">API Docs →</a>
  <a href="https://github.com/sebastienrousseau/oxml-json">GitHub</a>
  <a href="ASSURANCE-CASE/">Assurance Case</a>
  <a href="TESTING/">Testing</a>
</div>

</section>

## What's inside

<div class="grid cards" markdown>

- :material-code-json:{ .lg .middle } **Zero-dependency JSON**

    ---

    Compact JSON value enum (`Value`), recursive descent parser, and serialiser with zero external dependencies.

    [→ API Reference](https://docs.rs/oxml-json)

- :material-shield-check:{ .lg .middle } **Zero `unsafe` code**

    ---

    `#![forbid(unsafe_code)]` enforced at crate root. Clean, safe Rust with zero pointer gymnastics.

    [→ Assurance Case](ASSURANCE-CASE.md)

- :material-lightning-bolt:{ .lg .middle } **Built for JSON-RPC**

    ---

    Designed specifically for oxml's Language Server (`oxml-lsp`) and Model Context Protocol (`oxml-mcp`) servers.

    [→ Roadmap](ROADMAP.md)

- :material-check-decagram:{ .lg .middle } **Rigorous test suite**

    ---

    Comprehensive unit and fuzz tests verifying parsing limits, nesting depth bounds, and serialization roundtrips.

    [→ Testing Guide](TESTING.md)

</div>

## Quick start

Add `oxml-json` to your project:

```toml
[dependencies]
oxml-json = "0.0.10"
```

Parse and construct JSON values:

```rust
use oxml_json::Value;

fn main() -> Result<(), Box<dyn std.error.Error>> {
    let json_text = r#"{"name": "oxml", "safe": true, "version": "0.0.10"}"#;

    let value = Value::parse(json_text)?;
    assert_eq!(value["name"].as_str(), Some("oxml"));
    assert_eq!(value["safe"].as_bool(), Some(true));

    Ok(())
}
```

## Where to next

- [**API Documentation**](https://docs.rs/oxml-json) — Complete Rustdoc API reference on docs.rs.
- [**Assurance Case**](ASSURANCE-CASE.md) — Security posture, safety invariants, and dependency model.
- [**Testing**](TESTING.md) — Fuzzing, depth limit validation, and property testing.
- [**Roadmap**](ROADMAP.md) — Milestones and future optimizations.

## Current release

- Release notes: [GitHub Releases](https://github.com/sebastienrousseau/oxml-json/releases)
- Crates.io: [crates.io/crates/oxml-json](https://crates.io/crates/oxml-json)
- Repository: [sebastienrousseau/oxml-json](https://github.com/sebastienrousseau/oxml-json)
