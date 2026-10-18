// SPDX-License-Identifier: MIT OR Apache-2.0
//! Untrusted JSON must have bounded recursion and strict lexical syntax.

#[test]
fn nesting_is_bounded_for_arrays_and_objects() {
    for source in [
        format!("{}0{}", "[".repeat(257), "]".repeat(257)),
        format!("{}0{}", "{\"a\":".repeat(257), "}".repeat(257)),
    ] {
        let error = oxml_json::parse(&source).expect_err("nesting limit");
        assert!(error.contains("nesting limit"), "{error}");
    }
    let boundary = format!("{}0{}", "[".repeat(256), "]".repeat(256));
    assert!(oxml_json::parse(&boundary).is_ok());
}

#[test]
fn oversized_source_is_rejected_before_parsing() {
    let source = " ".repeat(16 * 1024 * 1024 + 1);
    let error = oxml_json::parse(&source).expect_err("source limit");
    assert!(error.contains("source limit"), "{error}");
}

#[test]
fn only_json_number_spellings_are_accepted() {
    for source in ["+1", "01", "-01", ".5", "1.", "1.e2", "00e1"] {
        assert!(oxml_json::parse(source).is_err(), "accepted {source}");
    }
    for source in ["0", "-0", "0.5", "1e+2", "1E-2", "-12.34e2"] {
        assert!(oxml_json::parse(source).is_ok(), "rejected {source}");
    }
}

#[test]
fn invalid_string_escapes_and_raw_controls_are_rejected() {
    for source in [r#""\q""#, r#""\x20""#, "\"raw\nline\"", "\"\u{0}\""] {
        assert!(oxml_json::parse(source).is_err(), "accepted {source:?}");
    }
    for source in [r#""\/\\\"\b\f\n\r\t""#, "\"é😀\""] {
        assert!(oxml_json::parse(source).is_ok(), "rejected {source:?}");
    }
}
