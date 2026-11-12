//! The model chip's response parsing, shared by the two shells: the one-child
//! shell folds responses in `events`, the chats shell in `chats`, and both
//! must fill the same [`AppState`](crate::state::AppState) fields from the
//! same replies (0.1.11).

use serde_json::Value;

use crate::state::{AppState, ModelChoice, ModelsStatus};

/// One Model object in pi's shape (`id`, `name`, `provider`, optional
/// `reasoning`) flattened for the chip. `None` when the value is not a model.
pub(crate) fn model_from_value(value: Option<&Value>) -> Option<ModelChoice> {
    let object = value?.as_object()?;
    let model_id = object.get("id")?.as_str()?.to_owned();
    Some(ModelChoice {
        provider: object.get("provider")?.as_str()?.to_owned(),
        model_id: model_id.clone(),
        name: object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(&model_id)
            .to_owned(),
        reasoning: object.get("reasoning").and_then(Value::as_bool) == Some(true),
    })
}

/// Applies one successful reply of the chip's four questions to the state.
/// Answers whether the command was one of them, so a caller can fall through
/// to its own handling for everything else.
pub(crate) fn apply_ok(state: &mut AppState, command: &str, data: &Value) -> bool {
    match command {
        "get_state" => {
            // The session reports its model and thinking level; the chip
            // renders both. Unknown fields stay unknown.
            state.model = model_from_value(data.get("model"));
            state.thinking_level = data
                .get("thinkingLevel")
                .and_then(Value::as_str)
                .map(str::to_owned);
        }
        "get_available_models" => {
            state.models = data
                .get("models")
                .and_then(Value::as_array)
                .map(|models| {
                    models
                        .iter()
                        .filter_map(|m| model_from_value(Some(m)))
                        .collect()
                })
                .unwrap_or_default();
            state.models_status = if state.models.is_empty() {
                ModelsStatus::Empty
            } else {
                ModelsStatus::Ready
            };
        }
        "set_model" => state.model = model_from_value(Some(data)),
        "get_available_thinking_levels" => {
            state.thinking_levels = data
                .as_array()
                .map(|levels| {
                    levels
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default();
        }
        _ => return false,
    }
    true
}

/// The failure of one of the chip's own asks, as both shells render it: the
/// list ask marks the status failed and notices, the level ask drops the
/// optimistic level back to unknown and notices.
pub(crate) fn apply_err(
    state: &mut AppState,
    command: &str,
    error: &str,
    notice: &mut dyn FnMut(&mut AppState, String),
) {
    match command {
        "get_available_models" => {
            state.models_status = ModelsStatus::Failed(error.to_owned());
            notice(state, format!("get_available_models failed: {error}"));
        }
        "set_thinking_level" => {
            state.thinking_level = None;
            notice(state, format!("set_thinking_level failed: {error}"));
        }
        _ => {}
    }
}
