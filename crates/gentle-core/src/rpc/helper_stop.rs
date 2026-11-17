//! Stopping one delegated helper over the desktop channel (helper-stop T3).
//!
//! The channel's subagents slice advertises exactly two methods, `list_subagents`
//! and `cancel_subagent`, and the cancel is the one this feature asks for. The
//! request is the contract's own: `params` is exactly `{ "id": "<helper id>" }`,
//! where the id is the one `list_subagents` reports - and the one
//! `gentle-agents.activity/v1` carries in a task's `summary.id`, because both are
//! the owner store's own `TaskRecord.id` (gentle-pi `lib/agents-rpc-publisher.ts`
//! and `lib/desktop-rpc-agents.ts` project the same record).
//!
//! The subtlety worth naming: an accepted cancel of a RUNNING helper answers
//! `ok: true` while the reported state is still `running` - the transition is
//! asynchronous and arrives later as an `agents_update` - so "accepted" is what
//! this module reports, never "already cancelled".
//!
//! Nothing here decides what the app does with an answer: every failure is a
//! typed [`StopOutcome`], so a caller can degrade silently (S7).

use serde_json::{json, Value};

use super::desktop::{
    DesktopChannel, DesktopClient, DesktopError, DesktopOutcome, NoAnswer, RequestError,
};

/// The method that cancels one subagent.
pub const CANCEL_SUBAGENT: &str = "cancel_subagent";

/// The `cancel_subagent` params: exactly the helper id.
///
/// The contract accepts only the `id` key - any other key, a non-string or an
/// empty id is `invalid_request` - so this builder writes nothing else.
pub fn cancel_subagent_params(helper_id: &str) -> Value {
    json!({ "id": helper_id })
}

/// What one stop attempt came to. Plain data: nothing here is an error the user
/// sees, and [`StopOutcome::Stopped`] is the only outcome worth an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopOutcome {
    /// The child accepted the cancel. A running helper's own transition follows
    /// through `agents_update`, so its reported state may still be `running`.
    Stopped,
    /// The child refused the request and says why (`not_found` for an unknown or
    /// foreign id, `agents_unavailable` when the owner is absent,
    /// `invalid_request` for a shape it does not accept).
    Rejected(DesktopError),
    /// The child's `capabilities` does not list `cancel_subagent`, so nothing was
    /// asked (an older child, or a host that does not implement the slice).
    Unsupported,
    /// No usable answer: the host is not interactive, the child never answered
    /// before the deadline, it closed, or its payload was not this contract's.
    NoAnswer(NoAnswer),
    /// The request broke a contract limit and never left.
    Invalid(RequestError),
}

/// Asks one chat's child to cancel `helper_id`, through a client that keeps that
/// chat's own request ids and cached capabilities.
///
/// The client's first call of a session is `capabilities`; this asks it when the
/// client has not resolved one yet, so a failed connection is retried on the next
/// stop instead of poisoning the chat. A client that still has no contract
/// answers [`NoAnswer::NotConnected`] and writes nothing, which is the channel's
/// own rule: support is never inferred.
pub fn stop_helper<C: DesktopChannel>(
    client: &mut DesktopClient<C>,
    helper_id: &str,
) -> StopOutcome {
    if client.capabilities().is_none() {
        client.connect();
    }
    match client.ask(CANCEL_SUBAGENT, cancel_subagent_params(helper_id)) {
        DesktopOutcome::Data { .. } => StopOutcome::Stopped,
        DesktopOutcome::Rejected { error, .. } => StopOutcome::Rejected(error),
        DesktopOutcome::Unsupported { .. } => StopOutcome::Unsupported,
        DesktopOutcome::NoAnswer(reason) => StopOutcome::NoAnswer(reason),
        DesktopOutcome::Invalid(error) => StopOutcome::Invalid(error),
    }
}
