//! The context reading one chat's shell reports: how much of the model's window
//! its session already holds (0.1.12, S5).
//!
//! pi does not push this. `get_session_stats` answers it and nothing else does
//! (rpc-mode.js:488-490), so a chat asks again when its turn settles and on
//! open, and the reading is the child's truth for that chat: the app stores it
//! beside the chat's own state and never computes one.
//!
//! Both numbers are legitimately absent: right after a compaction pi answers
//! `tokens: null` and `percent: null` for a moment. `None` therefore means "no
//! reading", never "zero", which is what lets the chip draw `-` instead of a
//! false `0%`.

use serde_json::Value;

/// How full the session's window is, as pi's `contextUsage` reports it
/// (`rpc-types.d.ts:303-306`). Every field is optional because every one of
/// them can be unknown either at the reply's level (an older child that does
/// not carry it) or at the field's (a just-compacted session).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContextUsage {
    /// The tokens the session's context holds, when pi counted them.
    pub tokens: Option<u64>,
    /// The model's window, the denominator of `percent`.
    pub context_window: Option<u64>,
    /// The share of the window in use, as pi computed it. Never rounded here:
    /// the view decides how many digits it draws.
    pub percent: Option<f64>,
}

/// Reads the `contextUsage` object of one successful `get_session_stats` reply.
///
/// `None` when the reply carries none - a child that answered without the
/// field, a `null` data, or an untyped answer - so the caller keeps whatever it
/// knew and never invents a reading. A `contextUsage` whose numbers are all
/// null is a real reading of "nothing yet" (`Some` with no fields), which is the
/// state right after a compaction.
pub(crate) fn from_reply(data: &Value) -> Option<ContextUsage> {
    let usage = data.get("contextUsage")?.as_object()?;
    Some(ContextUsage {
        tokens: usage.get("tokens").and_then(Value::as_u64),
        context_window: usage.get("contextWindow").and_then(Value::as_u64),
        percent: usage.get("percent").and_then(Value::as_f64),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_reply_carries_the_three_numbers() {
        let usage = from_reply(&json!({
            "contextUsage": {"tokens": 38000, "contextWindow": 200000, "percent": 19.0}
        }))
        .expect("a contextUsage object is a reading");
        assert_eq!(usage.tokens, Some(38_000));
        assert_eq!(usage.context_window, Some(200_000));
        assert_eq!(usage.percent, Some(19.0));
    }

    #[test]
    fn a_just_compacted_session_is_a_reading_with_no_numbers() {
        // pi answers both as null right after a compaction. That is a reading
        // ("nothing yet"), so the chip can draw `-` rather than a false 0%.
        let usage = from_reply(&json!({
            "contextUsage": {"tokens": null, "contextWindow": 200000, "percent": null}
        }))
        .expect("the object is there");
        assert_eq!(usage.tokens, None);
        assert_eq!(usage.percent, None);
        assert_eq!(usage.context_window, Some(200_000));
    }

    #[test]
    fn a_reply_without_the_field_is_no_reading_at_all() {
        // Unknown data, no data, no field: the app keeps what it knew instead
        // of showing a zero it was never told.
        assert_eq!(from_reply(&json!({})), None);
        assert_eq!(from_reply(&Value::Null), None);
        assert_eq!(from_reply(&json!({"contextUsage": null})), None);
        assert_eq!(from_reply(&json!({"contextUsage": 7})), None);
        assert_eq!(from_reply(&json!([1, 2])), None);
    }

    #[test]
    fn numbers_of_another_shape_are_unknown_and_never_guessed() {
        let usage = from_reply(&json!({
            "contextUsage": {"tokens": "38k", "contextWindow": -1, "percent": "19%"}
        }))
        .expect("the object is there");
        assert_eq!(usage.tokens, None);
        assert_eq!(usage.context_window, None);
        assert_eq!(usage.percent, None);
    }
}
