// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml-json. All rights reserved.

use super::*;

fn round_trip(text: &str) -> String {
    parse(text).expect("valid JSON").to_json()
}

#[test]
fn a_number_too_large_for_f64_is_refused() {
    // Grammatically valid JSON that `f64` cannot hold. Parsing it
    // to infinity and serialising that produced output which would
    // not parse -- the round trip silently broke.
    //
    // Fuzzing found it on `-3e805`. Both directions are checked
    // because the overflow is symmetric and only one sign was
    // reported.
    for text in ["1e999", "-3e805", "1e309"] {
        let err = parse(text).expect_err("should be refused");
        assert!(
            err.contains("out of range"),
            "{text} gave the wrong error: {err}"
        );
    }
    // The largest finite double still parses, so the check bounds
    // the value rather than the exponent's length.
    assert!(parse("1.7976931348623157e308").is_ok());
}

#[test]
fn scalars_round_trip() {
    assert_eq!(round_trip("null"), "null");
    assert_eq!(round_trip("true"), "true");
    assert_eq!(round_trip("false"), "false");
    assert_eq!(round_trip("\"hi\""), "\"hi\"");
}

#[test]
fn integral_numbers_print_without_a_fraction() {
    // MCP ids are compared by clients; `2.0` where `2` was sent is
    // a different JSON value to a strict client.
    assert_eq!(round_trip("2"), "2");
    assert_eq!(round_trip("-17"), "-17");
    assert_eq!(round_trip("0"), "0");
    assert_eq!(round_trip("1.5"), "1.5");
    assert_eq!(round_trip("1e3"), "1000");
}

#[test]
fn very_large_integral_numbers_do_not_wrap() {
    // Beyond 1e15 the i64 cast would be lossy, so the float path
    // must take over rather than silently truncating.
    let out = Json::Number(1e300).to_json();
    assert!(out.starts_with('1'), "{out}");
    assert!(parse(&out).is_ok(), "{out} did not re-parse");
}

#[test]
fn objects_serialise_in_a_stable_order() {
    let text = round_trip(r#"{"b":1,"a":2}"#);
    assert_eq!(text, r#"{"a":2,"b":1}"#);
}

#[test]
fn nesting_round_trips() {
    let src = r#"{"a":[1,[2,{"b":null}]],"c":{}}"#;
    assert_eq!(round_trip(src), src);
}

#[test]
fn string_escapes_decode() {
    let v = parse(r#""a\nb\tc\"d\\e\/f\r\b\f""#).expect("valid");
    assert_eq!(v.as_str(), Some("a\nb\tc\"d\\e/f\r\u{8}\u{c}"));
}

#[test]
fn control_characters_are_escaped_on_output() {
    // XML documents do contain these, and an unescaped control
    // character makes the whole response unparseable to the client.
    let out = Json::str("a\u{1}b\u{1f}").to_json();
    assert_eq!(out, r#""a\u0001b\u001f""#);
    assert_eq!(
        parse(&out).expect("re-parses").as_str(),
        Some("a\u{1}b\u{1f}")
    );
}

#[test]
fn surrogate_pairs_decode_to_one_character() {
    // Python's `json.dumps` escapes non-ASCII by default, so a
    // client written against it sends emoji this way.
    assert_eq!(
        parse(r#""\ud83d\ude00""#).expect("valid").as_str(),
        Some("\u{1f600}")
    );
    assert_eq!(
        parse(r#""a\ud83d\ude00b""#).expect("valid").as_str(),
        Some("a\u{1f600}b")
    );
}

#[test]
fn bmp_escapes_still_decode() {
    assert_eq!(
        parse(r#""\u00e9\u4e2d""#).expect("valid").as_str(),
        Some("é中")
    );
}

#[test]
fn lone_surrogates_are_rejected() {
    // Rather than panicking or emitting an invalid character.
    assert!(parse(r#""\ud83d""#).is_err());
    assert!(parse(r#""\ude00""#).is_err());
    assert!(parse(r#""\ud83dx""#).is_err());
    assert!(parse(r#""\ud83d\u0041""#).is_err());
}

#[test]
fn raw_utf8_passes_through_unescaped() {
    let src = "\"héllo 😀 中\"";
    assert_eq!(round_trip(src), src);
}

#[test]
fn malformed_input_is_an_error_not_a_panic() {
    for bad in [
        "",
        "{",
        "[",
        "{\"a\"}",
        "{\"a\":}",
        "[1,",
        "\"unterminated",
        "tru",
        "nul",
        "\"\\",
        "\"\\u00\"",
        "\"\\uZZZZ\"",
        "@",
        "{\"a\":1}trailing",
        "1 2",
    ] {
        assert!(parse(bad).is_err(), "`{bad}` should not parse");
    }
}

#[test]
fn whitespace_around_values_is_ignored() {
    assert_eq!(
        round_trip(" \n\t{ \"a\" : [ 1 , 2 ] }\r\n"),
        r#"{"a":[1,2]}"#
    );
}

#[test]
fn get_and_as_str_are_type_safe() {
    let v = parse(r#"{"a":"x","b":1}"#).expect("valid");
    assert_eq!(v.get("a").and_then(Json::as_str), Some("x"));
    assert_eq!(v.get("b").and_then(Json::as_str), None);
    assert!(v.get("missing").is_none());
    assert!(Json::Null.get("a").is_none());
    assert!(Json::Null.as_str().is_none());
}

#[test]
fn object_builder_matches_parsed_form() {
    let built =
        Json::object(vec![("a", Json::str("x")), ("b", Json::Number(1.0))]);
    assert_eq!(built.to_json(), r#"{"a":"x","b":1}"#);
}

#[test]
fn deep_nesting_does_not_overflow_the_stack() {
    let depth = 200;
    let src = format!("{}{}", "[".repeat(depth), "]".repeat(depth));
    assert!(parse(&src).is_ok());
}
