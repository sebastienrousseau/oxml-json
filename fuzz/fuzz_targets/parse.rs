#![no_main]
//! Arbitrary text must never panic the parser, and anything it accepts
//! must survive a round trip.
//!
//! This target moved here with the implementation. It previously ran
//! against `oxml-mcp`'s copy, which is the coverage this crate exists
//! to keep rather than lose in the move.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };

    let Ok(value) = oxml_json::parse(text) else {
        // Refusing malformed input is the expected outcome for random
        // bytes and is not interesting. What matters is that it
        // refused rather than panicked.
        return;
    };

    // Anything that parsed must serialise to something that parses
    // back to the same value. A serialiser that loses or mangles a
    // value it just accepted is worse than one that refuses it: the
    // loss is silent.
    let text = value.to_json();
    let again = oxml_json::parse(&text)
        .expect("output of to_json must parse");
    assert_eq!(value, again, "round trip changed the value: {text}");
});
