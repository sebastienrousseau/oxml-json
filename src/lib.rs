// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml-json. All rights reserved.

//! A small JSON value, parser and serialiser.
//!
//! This exists because two crates in the [oxml] suite speak JSON-RPC
//! — `oxml-mcp` over the Model Context Protocol and `oxml-lsp` over
//! the Language Server Protocol — and for a while each carried its own
//! copy of this file. A defect fixed in one would not have reached the
//! other, and nothing would have noticed.
//!
//! [oxml]: https://github.com/sebastienrousseau/oxml
//!
//! # Scope
//!
//! JSON-RPC messages are small and their shapes are fixed, so this
//! implements what those protocols need rather than everything the
//! grammar allows. It is not a general-purpose JSON library and does
//! not try to be: if you want `serde`, use `serde`.
//!
//! What it does do is refuse malformed input rather than guess at it,
//! and it is fuzzed on every pull request.
//!
//! # Numbers
//!
//! This implementation stores numbers as `f64`; JSON itself does not
//! require that representation. An integer beyond 2^53 does
//! may lose precision. Callers requiring larger exact integers must use a
//! different representation. The LSP transport uses this crate; MCP uses
//! its SDK's JSON implementation.
//!
//! ```
//! use oxml_json::{parse, Json};
//!
//! let value = parse(r#"{"jsonrpc":"2.0","id":1}"#).expect("valid JSON");
//! assert_eq!(value.get("jsonrpc").and_then(Json::as_str), Some("2.0"));
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// A JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    /// `null`
    Null,
    /// `true` or `false`
    Bool(bool),
    /// A number.
    Number(f64),
    /// A string.
    String(String),
    /// An array.
    Array(Vec<Json>),
    /// An object. Ordered so output is deterministic.
    Object(BTreeMap<String, Json>),
}

impl Json {
    /// Build an object from pairs.
    #[must_use]
    pub fn object(pairs: Vec<(&str, Self)>) -> Self {
        Self::Object(
            pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
        )
    }

    /// A string value.
    #[must_use]
    pub fn str(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    /// Look up a key on an object.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Object(m) => m.get(key),
            _ => None,
        }
    }

    /// The value as a string slice, if it is one.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Serialise to JSON text.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        self.write(&mut out);
        out
    }

    fn write(&self, out: &mut String) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(b) => {
                out.push_str(if *b { "true" } else { "false" });
            }
            Self::Number(n) => {
                // JSON has one number type; printing an integral
                // value as `2` rather than `2.0` is what every other
                // encoder does, and the bound keeps the cast exact.
                #[allow(clippy::cast_possible_truncation)]
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    let _ = write!(out, "{}", *n as i64);
                } else {
                    let _ = write!(out, "{n}");
                }
            }
            Self::String(s) => write_string(s, out),
            Self::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.write(out);
                }
                out.push(']');
            }
            Self::Object(map) => {
                out.push('{');
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_string(k, out);
                    out.push(':');
                    v.write(out);
                }
                out.push('}');
            }
        }
    }
}

fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Control characters must be escaped or the output is not
            // valid JSON, and XML documents do contain them.
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Parse JSON text.
///
/// # Errors
///
/// Returns a description if the input is not valid JSON, exceeds 16 MiB,
/// or contains more than 256 nested arrays and objects.
pub fn parse(input: &str) -> Result<Json, String> {
    if input.len() > 16 * 1024 * 1024 {
        return Err("JSON source limit exceeded (16 MiB)".to_owned());
    }
    let chars: Vec<char> = input.chars().collect();
    let mut p = Parser {
        chars: &chars,
        pos: 0,
        depth: 0,
    };
    p.ws();
    let v = p.value()?;
    p.ws();
    if p.pos < chars.len() {
        return Err("trailing input after JSON value".to_owned());
    }
    Ok(v)
}

struct Parser<'a> {
    chars: &'a [char],
    pos: usize,
    depth: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while matches!(self.chars.get(self.pos), Some(' ' | '\t' | '\n' | '\r'))
        {
            self.pos += 1;
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        let container = matches!(self.chars.get(self.pos), Some('{' | '['));
        if container && self.depth >= 256 {
            return Err("JSON nesting limit exceeded (256)".to_owned());
        }
        self.depth += usize::from(container);
        let result = self.value_inner();
        self.depth -= usize::from(container);
        result
    }

    fn value_inner(&mut self) -> Result<Json, String> {
        match self.chars.get(self.pos) {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => Ok(Json::String(self.string()?)),
            Some('t') => self.literal("true", Json::Bool(true)),
            Some('f') => self.literal("false", Json::Bool(false)),
            Some('n') => self.literal("null", Json::Null),
            Some(_) => self.number(),
            None => Err("unexpected end of JSON".to_owned()),
        }
    }

    fn literal(&mut self, word: &str, v: Json) -> Result<Json, String> {
        if self.chars[self.pos..]
            .starts_with(word.chars().collect::<Vec<_>>().as_slice())
        {
            self.pos += word.len();
            Ok(v)
        } else {
            Err(format!("expected `{word}`"))
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.pos += 1;
        let mut map = BTreeMap::new();
        self.ws();
        if self.chars.get(self.pos) == Some(&'}') {
            self.pos += 1;
            return Ok(Json::Object(map));
        }
        loop {
            self.ws();
            let key = self.string()?;
            self.ws();
            if self.chars.get(self.pos) != Some(&':') {
                return Err("expected `:` in object".to_owned());
            }
            self.pos += 1;
            let value = self.value()?;
            let _ = map.insert(key, value);
            self.ws();
            match self.chars.get(self.pos) {
                Some(',') => self.pos += 1,
                Some('}') => {
                    self.pos += 1;
                    return Ok(Json::Object(map));
                }
                _ => return Err("expected `,` or `}`".to_owned()),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.pos += 1;
        let mut items = Vec::new();
        self.ws();
        if self.chars.get(self.pos) == Some(&']') {
            self.pos += 1;
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.value()?);
            self.ws();
            match self.chars.get(self.pos) {
                Some(',') => self.pos += 1,
                Some(']') => {
                    self.pos += 1;
                    return Ok(Json::Array(items));
                }
                _ => return Err("expected `,` or `]`".to_owned()),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        if self.chars.get(self.pos) != Some(&'"') {
            return Err("expected a string".to_owned());
        }
        self.pos += 1;
        let mut out = String::new();
        loop {
            match self.chars.get(self.pos) {
                None => return Err("unterminated string".to_owned()),
                Some('"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some('\\') => {
                    self.pos += 1;
                    out.push(self.escape()?);
                }
                Some(c) if *c < '\u{20}' => {
                    return Err("unescaped control in string".to_owned());
                }
                Some(c) => {
                    out.push(*c);
                    self.pos += 1;
                }
            }
        }
    }

    fn escape(&mut self) -> Result<char, String> {
        let esc = *self.chars.get(self.pos).ok_or("trailing backslash")?;
        self.pos += 1;
        match esc {
            'u' => self.unicode_escape(),
            _ => simple_escape(esc)
                .ok_or_else(|| "invalid string escape".to_owned()),
        }
    }

    fn unicode_escape(&mut self) -> Result<char, String> {
        let cp = self.hex4()?;
        // A non-BMP character is escaped as a surrogate *pair*; neither
        // half is a `char` on its own. Python's `json.dumps` escapes
        // non-ASCII by default, so a client sending an emoji arrives
        // this way.
        let cp = if (0xD800..0xDC00).contains(&cp) {
            if !matches!(
                self.chars.get(self.pos..self.pos + 2),
                Some(['\\', 'u'])
            ) {
                return Err("lone high surrogate".to_owned());
            }
            self.pos += 2;
            let lo = self.hex4()?;
            if !(0xDC00..0xE000).contains(&lo) {
                return Err(
                    "high surrogate not followed by a low surrogate".to_owned()
                );
            }
            0x1_0000 + ((cp - 0xD800) << 10) + (lo - 0xDC00)
        } else if (0xDC00..0xE000).contains(&cp) {
            return Err("lone low surrogate".to_owned());
        } else {
            cp
        };
        char::from_u32(cp).ok_or_else(|| "bad code point".to_owned())
    }

    /// Read exactly four hex digits of a `\\u` escape.
    fn hex4(&mut self) -> Result<u32, String> {
        let hex: String = self
            .chars
            .get(self.pos..self.pos + 4)
            .ok_or("truncated \\u escape")?
            .iter()
            .collect();
        self.pos += 4;
        u32::from_str_radix(&hex, 16).map_err(|_| "bad \\u escape".to_owned())
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.pos;
        while matches!(
            self.chars.get(self.pos),
            Some(c) if c.is_ascii_digit()
                || matches!(c, '-' | '+' | '.' | 'e' | 'E')
        ) {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        if !valid_number(&text) {
            return Err(format!("invalid number `{text}`"));
        }
        let value: f64 = text
            .parse()
            .map_err(|_| format!("invalid number `{text}`"))?;
        // `"1e999"` is grammatically valid JSON and parses to
        // infinity, which JSON cannot express -- so serialising it
        // again produced output that would not parse. Refusing it here
        // is the honest answer for a parser that carries numbers as
        // `f64`: the alternative, writing `null`, silently changes the
        // value.
        //
        // Found by fuzzing, on `-3e805`, within ninety seconds of the
        // round-trip assertion being added.
        if !value.is_finite() {
            return Err(format!("number `{text}` is out of range for f64"));
        }
        Ok(Json::Number(value))
    }
}

fn simple_escape(escaped: char) -> Option<char> {
    match escaped {
        'n' => Some('\n'),
        't' => Some('\t'),
        'r' => Some('\r'),
        'b' => Some('\u{8}'),
        'f' => Some('\u{c}'),
        c if ['"', '\\', '/'].contains(&c) => Some(c),
        _ => None,
    }
}

fn digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

fn valid_mantissa(text: &str) -> bool {
    let (integer, fraction) = text
        .split_once('.')
        .map_or((text, None), |(integer, fraction)| {
            (integer, Some(fraction))
        });
    digits(integer)
        && !(integer.len() > 1 && integer.starts_with('0'))
        && fraction.is_none_or(digits)
}

fn valid_number(text: &str) -> bool {
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let mut parts = unsigned.split(['e', 'E']);
    let mantissa = parts.next().unwrap_or_default();
    let exponent = parts.next();
    parts.next().is_none()
        && valid_mantissa(mantissa)
        && exponent.is_none_or(|value| {
            let unsigned = value
                .strip_prefix('+')
                .or_else(|| value.strip_prefix('-'))
                .unwrap_or(value);
            digits(unsigned)
        })
}

#[cfg(test)]
#[path = "tests/basic.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/escapes.rs"]
mod escape_tests;
