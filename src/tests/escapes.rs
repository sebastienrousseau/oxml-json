// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml-json. All rights reserved.

use super::*;

#[test]
fn every_escape_survives_a_round_trip() {
    // Each of these has its own arm on both the reading and the
    // writing side, and a mismatch between the two corrupts the
    // document silently rather than failing.
    for s in [
        "back\\slash",
        "carriage\rreturn",
        "tab\there",
        "quote\"inside",
        "new\nline",
        "all\\ \" \n \r \t together",
    ] {
        let encoded = Json::str(s).to_json();
        assert_eq!(
            parse(&encoded).expect("re-parses").as_str(),
            Some(s),
            "round trip failed for {s:?} via {encoded}"
        );
    }
}

#[test]
fn a_structural_error_names_what_was_expected() {
    // A bare "invalid JSON" would give a client nothing to act on.
    let e = parse(r#"{"a":1 "b":2}"#).expect_err("malformed");
    assert!(e.contains(',') || e.contains('}'), "{e}");

    let e = parse("[1 2]").expect_err("malformed");
    assert!(e.contains(',') || e.contains(']'), "{e}");
}
