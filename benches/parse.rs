// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml-json. All rights reserved.

//! What parsing and serialising cost, by document shape.
//!
//! This exists to answer one question the roadmap asks and could not
//! previously settle: **every string in a parsed document is owned, so
//! is the copy per string a cost that matters for the bodies this
//! crate actually sees?** Those bodies are JSON-RPC messages for
//! `oxml-mcp` and `oxml-lsp` — a request of a few hundred bytes, or a
//! `tools/list` reply of a few kilobytes.
//!
//! The shapes below are chosen to answer it. `string-heavy` and
//! `number-heavy` hold the same number of entries but differ in how
//! much of each entry is string content; they are *not* the same byte
//! size, so the comparison is normalised per byte rather than taken
//! directly.
//!
//! No criterion. This crate has no dependencies, which is most of the
//! reason to use it, and a benchmark is not worth spending that on.
//! The estimator is the fastest of many runs rather than a mean: the
//! fastest run is the one least perturbed by whatever else the machine
//! was doing, and a mean mostly measures that.
//!
//! Absolute figures describe the machine as much as the code. Compare
//! runs, not numbers.

use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Instant;

/// A JSON-RPC request of the shape `oxml-mcp` receives.
fn mcp_request() -> String {
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"xml_query","arguments":{"xml":"<catalogue><book lang=\"en\"><title>Dune</title></book></catalogue>","xpath":"//book/title"}}}"#.to_owned()
}

/// A `tools/list` reply: four tools, each with a nested schema.
fn tools_list(tools: usize) -> String {
    let mut s = String::from(r#"{"jsonrpc":"2.0","id":2,"result":{"tools":["#);
    for i in 0..tools {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(
            s,
            r#"{{"name":"xml_tool_{i}","description":"Evaluate an XPath 1.0 expression against a document and return the matching values. Use this instead of reading a large document into context.","inputSchema":{{"type":"object","properties":{{"xml":{{"type":"string","description":"The XML document"}},"xpath":{{"type":"string","description":"An XPath 1.0 expression"}}}},"required":["xml","xpath"]}}}}"#
        );
    }
    s.push_str("]}}");
    s
}

/// `n` entries whose content is almost entirely string.
fn string_heavy(n: usize) -> String {
    let mut s = String::from("[");
    for i in 0..n {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(
            s,
            r#"{{"name":"entry number {i} with a reasonably long value"}}"#
        );
    }
    s.push(']');
    s
}

/// The same shape and entry count, but numeric rather than textual.
///
/// The point of the pair: they differ in how much of each entry is
/// string content, so the per-byte gap between them is what owning
/// the strings costs.
fn number_heavy(n: usize) -> String {
    let mut s = String::from("[");
    for i in 0..n {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(s, r#"{{"value{i}":{}.{i}}}"#, i * 7919);
    }
    s.push(']');
    s
}

/// Nesting, to keep an eye on the recursion bound.
fn deeply_nested(depth: usize) -> String {
    let mut s = String::new();
    for _ in 0..depth {
        s.push_str(r#"{"a":"#);
    }
    s.push('1');
    for _ in 0..depth {
        s.push('}');
    }
    s
}

/// Bytes as a float, for the throughput arithmetic.
///
/// `usize` to `f64` is lossy above 2^53. These documents are
/// kilobytes, so the cast is exact -- but the lint is right in
/// general and the reason it does not apply here belongs in writing.
#[allow(clippy::cast_precision_loss)]
fn as_f64(n: usize) -> f64 {
    n as f64
}

fn fastest(rounds: usize, mut f: impl FnMut()) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..rounds {
        let start = Instant::now();
        f();
        best = best.min(start.elapsed().as_secs_f64());
    }
    best
}

fn main() {
    let cases: Vec<(String, String)> = vec![
        ("mcp request".to_owned(), mcp_request()),
        ("tools/list reply".to_owned(), tools_list(4)),
        ("string-heavy (500)".to_owned(), string_heavy(500)),
        ("number-heavy (500)".to_owned(), number_heavy(500)),
        ("nested (64 deep)".to_owned(), deeply_nested(64)),
    ];

    println!("parse, fastest of 200 runs\n");
    let mut parsed = Vec::new();
    for (name, doc) in &cases {
        let secs = fastest(200, || {
            let _ = black_box(oxml_json::parse(black_box(doc)));
        });
        let len = doc.len();
        let len_f = as_f64(len);
        println!(
            "  {name:<22} {:>8.3} µs  ({len:>6} bytes, {:>6.1} MB/s)",
            secs * 1e6,
            len_f / secs / 1e6
        );
        parsed.push((name.clone(), oxml_json::parse(doc).expect("valid")));
    }

    println!("\nto_json, fastest of 200 runs\n");
    for (name, value) in &parsed {
        let secs = fastest(200, || {
            let _ = black_box(value.to_json());
        });
        println!("  {name:<22} {:>8.3} µs", secs * 1e6);
    }

    // The roadmap's question, answered rather than left open. If
    // owning every string dominated, the string-heavy shape would be
    // markedly slower per byte than the numeric one.
    let s = parsed.iter().find(|(n, _)| n == "string-heavy (500)");
    let n = parsed.iter().find(|(n, _)| n == "number-heavy (500)");
    if let (Some((_, sv)), Some((_, nv))) = (s, n) {
        let sd = string_heavy(500);
        let nd = number_heavy(500);
        let st = fastest(200, || {
            let _ = black_box(oxml_json::parse(black_box(&sd)));
        }) / as_f64(sd.len());
        let nt = fastest(200, || {
            let _ = black_box(oxml_json::parse(black_box(&nd)));
        }) / as_f64(nd.len());
        println!(
            "\nper byte: string-heavy {:.2} ns, number-heavy {:.2} ns, ratio {:.2}x",
            st * 1e9,
            nt * 1e9,
            st / nt
        );
        println!(
            "  (values kept live so the comparison is not optimised away: {} vs {} bytes)",
            sv.to_json().len(),
            nv.to_json().len()
        );
    }
}
