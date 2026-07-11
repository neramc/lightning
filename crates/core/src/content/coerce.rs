// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! The explicit coercion table (CLAUDE.md §6.2).
//!
//! Every allowed conversion is spelled out here and unit-tested. If a pair is
//! not in this table, the coercion fails with [`CoerceError::Unsupported`] —
//! actions must never invent implicit conversions.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use super::{Content, ContentKind};
use crate::error::CoerceError;

/// Image file extensions [`ContentKind::File`] → [`ContentKind::Image`]
/// accepts ("File→Image when decodable").
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff", "tif"];

fn number_to_text(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

fn parse_error(value: &Content, to: ContentKind) -> CoerceError {
    CoerceError::Parse { value: value.preview(60), to }
}

/// Coerce `value` to `target` per the explicit table.
pub fn coerce(value: &Content, target: ContentKind) -> Result<Content, CoerceError> {
    if value.kind() == target {
        return Ok(value.clone());
    }

    match (value, target) {
        // ── to Text ────────────────────────────────────────────────────────
        (Content::Nothing, ContentKind::Text) => Ok(Content::Text(String::new())),
        (Content::Number(n), ContentKind::Text) => Ok(Content::Text(number_to_text(*n))),
        (Content::Boolean(b), ContentKind::Text) => Ok(Content::Text(b.to_string())),
        (Content::Date(d), ContentKind::Text) => Ok(Content::Text(d.to_rfc3339())),
        (Content::Url(u), ContentKind::Text) => Ok(Content::Text(u.clone())),
        (Content::File(p) | Content::Image(p), ContentKind::Text) => {
            Ok(Content::Text(p.display().to_string()))
        }
        (Content::RichText { plain, .. }, ContentKind::Text) => Ok(Content::Text(plain.clone())),
        (Content::Error { message }, ContentKind::Text) => Ok(Content::Text(message.clone())),
        // List→Text: newline-join of item texts.
        (Content::List(items), ContentKind::Text) => {
            let mut lines = Vec::with_capacity(items.len());
            for item in items {
                lines.push(item.as_text()?);
            }
            Ok(Content::Text(lines.join("\n")))
        }
        // Dictionary→Text: canonical JSON.
        (dict @ Content::Dictionary(_), ContentKind::Text) => {
            let json = serde_json::to_string_pretty(&content_to_json(dict))
                .map_err(|_| parse_error(value, ContentKind::Text))?;
            Ok(Content::Text(json))
        }

        // ── to Number ──────────────────────────────────────────────────────
        (Content::Text(s), ContentKind::Number) => s
            .trim()
            .parse::<f64>()
            .map(Content::Number)
            .map_err(|_| parse_error(value, ContentKind::Number)),
        (Content::Boolean(b), ContentKind::Number) => {
            Ok(Content::Number(if *b { 1.0 } else { 0.0 }))
        }
        (Content::Date(d), ContentKind::Number) => {
            Ok(Content::Number(d.timestamp_millis() as f64 / 1000.0))
        }

        // ── to Boolean ─────────────────────────────────────────────────────
        (Content::Text(s), ContentKind::Boolean) => match s.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(Content::Boolean(true)),
            "false" | "no" | "0" => Ok(Content::Boolean(false)),
            _ => Err(parse_error(value, ContentKind::Boolean)),
        },
        (Content::Number(n), ContentKind::Boolean) => Ok(Content::Boolean(*n != 0.0)),

        // ── to Date ────────────────────────────────────────────────────────
        (Content::Text(s), ContentKind::Date) => DateTime::parse_from_rfc3339(s.trim())
            .map(|d| Content::Date(d.with_timezone(&Utc)))
            .map_err(|_| parse_error(value, ContentKind::Date)),

        // ── to URL ─────────────────────────────────────────────────────────
        (Content::Text(s), ContentKind::Url) => {
            let trimmed = s.trim();
            if trimmed.contains("://") && !trimmed.contains(char::is_whitespace) {
                Ok(Content::Url(trimmed.to_owned()))
            } else {
                Err(parse_error(value, ContentKind::Url))
            }
        }

        // ── to File / Image ────────────────────────────────────────────────
        (Content::Image(p), ContentKind::File) => Ok(Content::File(p.clone())),
        (Content::File(p), ContentKind::Image) => {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or_default();
            if IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()) {
                Ok(Content::Image(p.clone()))
            } else {
                Err(parse_error(value, ContentKind::Image))
            }
        }

        // ── to RichText ────────────────────────────────────────────────────
        (Content::Text(s), ContentKind::RichText) => Ok(Content::RichText {
            html: html_escape(s),
            plain: s.clone(),
        }),

        // ── to List: anything wraps into a single-item list ────────────────
        (other, ContentKind::List) => Ok(Content::List(vec![other.clone()])),

        // Everything else is not in the table.
        _ => Err(CoerceError::Unsupported { from: value.kind(), to: target }),
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn content_to_json(value: &Content) -> serde_json::Value {
    match value {
        Content::Nothing => serde_json::Value::Null,
        Content::Text(s) => serde_json::Value::String(s.clone()),
        // Integers stay integers so JSON round-trips (1 must not become 1.0).
        Content::Number(n) if n.fract() == 0.0 && n.abs() < 9e15 => {
            serde_json::Value::from(*n as i64)
        }
        Content::Number(n) => serde_json::Number::from_f64(*n)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Content::Boolean(b) => serde_json::Value::Bool(*b),
        Content::Date(d) => serde_json::Value::String(d.to_rfc3339()),
        Content::List(items) => {
            serde_json::Value::Array(items.iter().map(content_to_json).collect())
        }
        Content::Dictionary(map) => serde_json::Value::Object(
            map.iter().map(|(k, v)| (k.clone(), content_to_json(v))).collect(),
        ),
        Content::File(p) | Content::Image(p) => {
            serde_json::Value::String(p.display().to_string())
        }
        Content::Url(u) => serde_json::Value::String(u.clone()),
        Content::RichText { plain, .. } => serde_json::Value::String(plain.clone()),
        Content::Error { message } => serde_json::Value::String(format!("error: {message}")),
    }
}

/// Convert JSON into [`Content`] (used by the scripting bridge and dictionary
/// actions). Numbers become `Number`, objects become `Dictionary`.
#[must_use]
pub fn json_to_content(value: &serde_json::Value) -> Content {
    match value {
        serde_json::Value::Null => Content::Nothing,
        serde_json::Value::Bool(b) => Content::Boolean(*b),
        serde_json::Value::Number(n) => Content::Number(n.as_f64().unwrap_or(f64::NAN)),
        serde_json::Value::String(s) => Content::Text(s.clone()),
        serde_json::Value::Array(items) => {
            Content::List(items.iter().map(json_to_content).collect())
        }
        serde_json::Value::Object(map) => Content::Dictionary(
            map.iter().map(|(k, v)| (k.clone(), json_to_content(v))).collect::<BTreeMap<_, _>>(),
        ),
    }
}

/// Convert [`Content`] into JSON (inverse of [`json_to_content`], lossy for
/// file/date types which become strings).
#[must_use]
pub fn content_to_json_value(value: &Content) -> serde_json::Value {
    content_to_json(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn number_to_text_trims_integer_fraction() {
        assert!(matches!(
            coerce(&Content::Number(42.0), ContentKind::Text),
            Ok(Content::Text(s)) if s == "42"
        ));
        assert!(matches!(
            coerce(&Content::Number(1.5), ContentKind::Text),
            Ok(Content::Text(s)) if s == "1.5"
        ));
    }

    #[test]
    fn text_to_number_parses_and_rejects() {
        assert!(matches!(
            coerce(&Content::Text(" 3.25 ".into()), ContentKind::Number),
            Ok(Content::Number(n)) if (n - 3.25).abs() < f64::EPSILON
        ));
        assert!(coerce(&Content::Text("abc".into()), ContentKind::Number).is_err());
    }

    #[test]
    fn list_to_text_joins_lines() {
        let list = Content::List(vec![Content::Text("a".into()), Content::Number(2.0)]);
        assert!(matches!(
            coerce(&list, ContentKind::Text),
            Ok(Content::Text(s)) if s == "a\n2"
        ));
    }

    #[test]
    fn file_to_image_requires_decodable_extension() {
        let png = Content::File(PathBuf::from("/tmp/pic.PNG"));
        assert!(matches!(coerce(&png, ContentKind::Image), Ok(Content::Image(_))));
        let doc = Content::File(PathBuf::from("/tmp/notes.txt"));
        assert!(coerce(&doc, ContentKind::Image).is_err());
    }

    #[test]
    fn anything_wraps_into_list() {
        assert!(matches!(
            coerce(&Content::Boolean(true), ContentKind::List),
            Ok(Content::List(items)) if items.len() == 1
        ));
    }

    #[test]
    fn unlisted_pairs_are_rejected() {
        // Date → Boolean is not in the table and must stay that way unless
        // added explicitly with tests.
        let date = Content::Date(Utc::now());
        assert!(matches!(
            coerce(&date, ContentKind::Boolean),
            Err(CoerceError::Unsupported { .. })
        ));
    }

    #[test]
    fn url_requires_a_scheme() {
        assert!(coerce(&Content::Text("https://example.com".into()), ContentKind::Url).is_ok());
        assert!(coerce(&Content::Text("not a url".into()), ContentKind::Url).is_err());
    }

    #[test]
    fn json_round_trip_preserves_structure() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"a": [1, true, "x"], "b": null}"#).expect("valid json");
        let content = json_to_content(&json);
        assert_eq!(content_to_json_value(&content), json);
    }
}
