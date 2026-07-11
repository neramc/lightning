// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! The typed content model (CLAUDE.md §6.2).
//!
//! Every action consumes and produces typed [`Content`]. Each step's output
//! is addressable downstream as a magic variable; named variables exist via
//! Set/Get Variable. Type conversions go through the **explicit** coercion
//! table in [`coerce`] — never implicit, never silently lossy.

pub mod coerce;

use std::collections::BTreeMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::CoerceError;

/// A typed value flowing between steps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum Content {
    /// The absence of a value (e.g. output of the Nothing action).
    Nothing,
    /// Plain text.
    Text(String),
    /// A double-precision number.
    Number(f64),
    /// A boolean.
    Boolean(bool),
    /// A point in time (UTC; rendered per user locale in the UI).
    Date(DateTime<Utc>),
    /// An ordered list of content values.
    List(Vec<Content>),
    /// String-keyed dictionary.
    Dictionary(BTreeMap<String, Content>),
    /// A reference to a file on disk.
    File(PathBuf),
    /// A reference to an image file on disk.
    Image(PathBuf),
    /// A URL.
    Url(String),
    /// Rich text carried as HTML plus its plain-text projection.
    RichText {
        /// HTML form.
        html: String,
        /// Plain-text form.
        plain: String,
    },
    /// An error value (produced when a step fails under the Continue policy).
    Error {
        /// Human-readable message.
        message: String,
    },
}

/// The kind (type tag) of a [`Content`] value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentKind {
    /// See [`Content::Nothing`].
    Nothing,
    /// See [`Content::Text`].
    Text,
    /// See [`Content::Number`].
    Number,
    /// See [`Content::Boolean`].
    Boolean,
    /// See [`Content::Date`].
    Date,
    /// See [`Content::List`].
    List,
    /// See [`Content::Dictionary`].
    Dictionary,
    /// See [`Content::File`].
    File,
    /// See [`Content::Image`].
    Image,
    /// See [`Content::Url`].
    Url,
    /// See [`Content::RichText`].
    RichText,
    /// See [`Content::Error`].
    Error,
}

impl std::fmt::Display for ContentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ContentKind::Nothing => "Nothing",
            ContentKind::Text => "Text",
            ContentKind::Number => "Number",
            ContentKind::Boolean => "Boolean",
            ContentKind::Date => "Date",
            ContentKind::List => "List",
            ContentKind::Dictionary => "Dictionary",
            ContentKind::File => "File",
            ContentKind::Image => "Image",
            ContentKind::Url => "URL",
            ContentKind::RichText => "Rich Text",
            ContentKind::Error => "Error",
        };
        f.write_str(name)
    }
}

impl Content {
    /// The kind of this value.
    #[must_use]
    pub fn kind(&self) -> ContentKind {
        match self {
            Content::Nothing => ContentKind::Nothing,
            Content::Text(_) => ContentKind::Text,
            Content::Number(_) => ContentKind::Number,
            Content::Boolean(_) => ContentKind::Boolean,
            Content::Date(_) => ContentKind::Date,
            Content::List(_) => ContentKind::List,
            Content::Dictionary(_) => ContentKind::Dictionary,
            Content::File(_) => ContentKind::File,
            Content::Image(_) => ContentKind::Image,
            Content::Url(_) => ContentKind::Url,
            Content::RichText { .. } => ContentKind::RichText,
            Content::Error { .. } => ContentKind::Error,
        }
    }

    /// Coerce this value to `target` via the explicit table.
    pub fn coerce_to(&self, target: ContentKind) -> Result<Content, CoerceError> {
        coerce::coerce(self, target)
    }

    /// Coerced text form.
    pub fn as_text(&self) -> Result<String, CoerceError> {
        match self.coerce_to(ContentKind::Text)? {
            Content::Text(s) => Ok(s),
            other => Err(CoerceError::Parse {
                value: format!("{other:?}"),
                to: ContentKind::Text,
            }),
        }
    }

    /// Coerced numeric form.
    pub fn as_number(&self) -> Result<f64, CoerceError> {
        match self.coerce_to(ContentKind::Number)? {
            Content::Number(n) => Ok(n),
            other => Err(CoerceError::Parse {
                value: format!("{other:?}"),
                to: ContentKind::Number,
            }),
        }
    }

    /// Coerced boolean form.
    pub fn as_boolean(&self) -> Result<bool, CoerceError> {
        match self.coerce_to(ContentKind::Boolean)? {
            Content::Boolean(b) => Ok(b),
            other => Err(CoerceError::Parse {
                value: format!("{other:?}"),
                to: ContentKind::Boolean,
            }),
        }
    }

    /// Items for Repeat-with-Each semantics: a `List` yields its items,
    /// `Nothing` yields no items, anything else is a single-item list.
    #[must_use]
    pub fn into_items(self) -> Vec<Content> {
        match self {
            Content::List(items) => items,
            Content::Nothing => Vec::new(),
            other => vec![other],
        }
    }

    /// A short, single-line preview for run progress events and logs.
    #[must_use]
    pub fn preview(&self, max_chars: usize) -> String {
        let full = match self {
            Content::Nothing => String::new(),
            Content::List(items) => format!("[{} items]", items.len()),
            Content::Dictionary(map) => format!("{{{} keys}}", map.len()),
            Content::Error { message } => format!("error: {message}"),
            other => other.as_text().unwrap_or_else(|_| other.kind().to_string()),
        };
        let mut line = full.replace(['\n', '\r'], " ");
        if line.chars().count() > max_chars {
            line = line
                .chars()
                .take(max_chars.saturating_sub(1))
                .collect::<String>()
                + "…";
        }
        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_shape_is_tagged() {
        let json = serde_json::to_value(Content::Text("hi".into())).expect("serialize");
        assert_eq!(json["type"], "text");
        assert_eq!(json["value"], "hi");
        let nothing = serde_json::to_value(Content::Nothing).expect("serialize");
        assert_eq!(nothing["type"], "nothing");
    }

    #[test]
    fn into_items_wraps_scalars_and_flattens_lists() {
        assert_eq!(Content::Nothing.into_items(), vec![]);
        assert_eq!(
            Content::Number(1.0).into_items(),
            vec![Content::Number(1.0)]
        );
        let list = Content::List(vec![Content::Text("a".into()), Content::Text("b".into())]);
        assert_eq!(list.into_items().len(), 2);
    }

    #[test]
    fn preview_truncates_and_flattens_newlines() {
        let text = Content::Text("line one\nline two that is quite long".into());
        let p = text.preview(12);
        assert!(p.chars().count() <= 12);
        assert!(!p.contains('\n'));
    }
}
