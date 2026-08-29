// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml-json. All rights reserved.

//! Reading a JSON-RPC request and building its reply.
//!
//! Run with:
//!
//! ```text
//! cargo run --example parse_and_build
//! ```

use oxml_json::{Json, parse};

fn main() {
    let request = r#"{"jsonrpc":"2.0","id":7,"method":"initialize","params":{"trace":"off"}}"#;

    let value = parse(request).expect("the request is valid JSON");

    // `get` walks objects; it returns `None` for a missing key and for
    // a value that is not an object, so chains do not need a type check
    // at every step.
    assert_eq!(
        value.get("method").and_then(Json::as_str),
        Some("initialize")
    );
    assert_eq!(
        value
            .get("params")
            .and_then(|p| p.get("trace"))
            .and_then(Json::as_str),
        Some("off")
    );

    // Building a reply. Object keys come back out in sorted order,
    // which makes serialised output comparable in a test without
    // parsing it again.
    let reply = Json::object(vec![
        ("jsonrpc", Json::str("2.0")),
        ("id", value.get("id").cloned().unwrap_or(Json::Null)),
        (
            "result",
            Json::object(vec![("capabilities", Json::object(vec![]))]),
        ),
    ]);

    println!("{}", reply.to_json());

    // Malformed input is an error with a message, not a panic and not
    // a silent default.
    let bad = parse("{\"unterminated\": ");
    println!("malformed input: {}", bad.unwrap_err());
}
