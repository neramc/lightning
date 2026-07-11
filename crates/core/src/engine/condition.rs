// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Condition evaluation for If / While steps.

use crate::content::{Content, ContentKind};
use crate::error::ActionError;

/// Comparison operators available on If / While steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CondOp {
    /// Equal (numeric when both sides coerce to numbers, else text).
    Eq,
    /// Not equal.
    Neq,
    /// Strictly greater than (numeric).
    Gt,
    /// Strictly less than (numeric).
    Lt,
    /// Text/list containment.
    Contains,
    /// The value exists and is non-empty.
    HasValue,
}

impl CondOp {
    /// Parse the stable id used in step params.
    #[must_use]
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "equals" => Some(CondOp::Eq),
            "notEquals" => Some(CondOp::Neq),
            "greaterThan" => Some(CondOp::Gt),
            "lessThan" => Some(CondOp::Lt),
            "contains" => Some(CondOp::Contains),
            "hasValue" => Some(CondOp::HasValue),
            _ => None,
        }
    }

    /// Whether this operator needs a right-hand side.
    #[must_use]
    pub const fn needs_rhs(self) -> bool {
        !matches!(self, CondOp::HasValue)
    }
}

/// Evaluate `lhs <op> rhs`.
pub fn evaluate(op: CondOp, lhs: &Content, rhs: Option<&Content>) -> Result<bool, ActionError> {
    let rhs = if op.needs_rhs() {
        Some(rhs.ok_or_else(|| ActionError::InvalidParam {
            param: "value".into(),
            message: format!("{op:?} needs a comparison value"),
        })?)
    } else {
        None
    };

    match op {
        CondOp::HasValue => Ok(has_value(lhs)),
        CondOp::Eq => Ok(loose_equals(lhs, rhs.expect("rhs required"))),
        CondOp::Neq => Ok(!loose_equals(lhs, rhs.expect("rhs required"))),
        CondOp::Gt => {
            let (a, b) = numeric_pair(lhs, rhs.expect("rhs required"))?;
            Ok(a > b)
        }
        CondOp::Lt => {
            let (a, b) = numeric_pair(lhs, rhs.expect("rhs required"))?;
            Ok(a < b)
        }
        CondOp::Contains => {
            let needle = rhs.expect("rhs required").as_text()?;
            match lhs {
                Content::List(items) => {
                    for item in items {
                        if item.as_text().map(|t| t == needle).unwrap_or(false) {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                other => Ok(other.as_text()?.contains(&needle)),
            }
        }
    }
}

fn has_value(value: &Content) -> bool {
    match value {
        Content::Nothing => false,
        Content::Text(s) => !s.is_empty(),
        Content::List(items) => !items.is_empty(),
        Content::Dictionary(map) => !map.is_empty(),
        _ => true,
    }
}

fn loose_equals(lhs: &Content, rhs: &Content) -> bool {
    // Numeric comparison when both sides coerce; else text comparison; a
    // failed coercion simply means "not equal", never an error.
    if let (Ok(a), Ok(b)) = (
        lhs.coerce_to(ContentKind::Number),
        rhs.coerce_to(ContentKind::Number),
    ) && let (Content::Number(a), Content::Number(b)) = (a, b)
    {
        return (a - b).abs() < f64::EPSILON;
    }
    match (lhs.as_text(), rhs.as_text()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn numeric_pair(lhs: &Content, rhs: &Content) -> Result<(f64, f64), ActionError> {
    Ok((lhs.as_number()?, rhs.as_number()?))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn equals_is_numeric_when_possible() {
        let lhs = Content::Text("42".into());
        let rhs = Content::Number(42.0);
        assert!(evaluate(CondOp::Eq, &lhs, Some(&rhs)).unwrap());
        assert!(!evaluate(CondOp::Neq, &lhs, Some(&rhs)).unwrap());
    }

    #[test]
    fn greater_than_requires_numbers() {
        let lhs = Content::Text("abc".into());
        assert!(evaluate(CondOp::Gt, &lhs, Some(&Content::Number(1.0))).is_err());
        assert!(
            evaluate(
                CondOp::Gt,
                &Content::Number(2.0),
                Some(&Content::Number(1.0))
            )
            .unwrap()
        );
    }

    #[test]
    fn contains_works_on_text_and_lists() {
        let text = Content::Text("hello world".into());
        assert!(
            evaluate(
                CondOp::Contains,
                &text,
                Some(&Content::Text("world".into()))
            )
            .unwrap()
        );
        let list = Content::List(vec![Content::Number(1.0), Content::Text("x".into())]);
        assert!(evaluate(CondOp::Contains, &list, Some(&Content::Text("1".into()))).unwrap());
        assert!(!evaluate(CondOp::Contains, &list, Some(&Content::Text("y".into()))).unwrap());
    }

    #[test]
    fn has_value_semantics() {
        assert!(!evaluate(CondOp::HasValue, &Content::Nothing, None).unwrap());
        assert!(!evaluate(CondOp::HasValue, &Content::Text(String::new()), None).unwrap());
        assert!(evaluate(CondOp::HasValue, &Content::Boolean(false), None).unwrap());
    }

    #[test]
    fn missing_rhs_is_an_invalid_param() {
        assert!(matches!(
            evaluate(CondOp::Eq, &Content::Nothing, None),
            Err(ActionError::InvalidParam { .. })
        ));
    }
}
