//! Running one per-chat command against the host.

use std::path::PathBuf;

use gentle_app::{ChatCommand, ChatId};
use gentle_backend::chat_host::Opened;
use gentle_core::i18n::{messages, Language};
use gentle_core::rpc::RpcCommand;

use super::{ChainWriteResult, ChatConfigRead, ChatProfileApply, ChatRuntime, ModeWriteResult};

/// What running a [`ChatCommand`] came to. Plain data: a failure is a message
/// for the shell to show, never a panic and never fatal for the whole app.
///
/// Not `Eq`: a read of a chat's profile carries the owner's own answer for each
/// store, which is compared but not totally ordered.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// The chat's child is up (started now, or already was).
    Opened { chat: ChatId, how: Opened },
    /// The child could not be started or the live cap left no room.
    OpenFailed { chat: ChatId, message: String },
    /// The command reached the chat's child.
    Sent,
    /// The command did not reach a child (none, still starting, or a broken pipe).
    SendFailed { chat: ChatId, message: String },
    /// The chat's child is gone, because this command stopped it (or because it
    /// had none). The chat itself is unchanged: its next open starts a fresh
    /// child through `--session` (0.1.14).
    Closed { chat: ChatId },
    /// The chat's child accepted the stop of one helper (helper-stop S2): that
    /// helper is cancelled from now on, and the chat keeps running. The child's
    /// own state may still say `running` for an instant - an accepted cancel of a
    /// running helper transitions asynchronously - so the acceptance, not the
    /// reported state, is what this says.
    Stopped { chat: ChatId, helper_id: String },
    /// No stop happened, and nothing is wrong (helper-stop S7): the chat has no
    /// child, its host is not interactive, the child never answered before the
    /// deadline, it closed, it refused (`not_found`), or its capabilities do not
    /// list the method. Today's behaviour: the helper keeps the mark it had, and
    /// the shell folds nothing.
    NotStopped { chat: ChatId, helper_id: String },
    /// The host's working mark was set for that chat, or a chat the host does
    /// not know was named (nothing to mark, nothing to answer).
    Working { chat: ChatId, working: bool },
    /// One chat's profile configuration, read through that chat's own child
    /// (profiles-editor T2b, S9): the three stores each carry their own answer,
    /// so one that failed never hides another. A read that could not be made is
    /// what [`StoreRead::Unread`](super::StoreRead::Unread) and
    /// [`StoreRead::Skipped`](super::StoreRead::Skipped) say, and today's
    /// read-only pane stays as it was (S6).
    ChatProfile {
        chat: ChatId,
        /// Boxed: the three typed stores make this variant far larger than the
        /// rest of the enum, and every `Outcome` pays for its largest variant.
        read: Box<ChatConfigRead>,
    },
    /// One profile's orchestration mode was saved over that chat's own child, or
    /// it was not and this is why (profiles-editor T3-mode, S2, S3). The write is
    /// followed by the same three reads a plain read makes, so the page draws
    /// the store's truth instead of what it hoped for.
    ChatMode {
        chat: ChatId,
        /// What the attempt came to: saved, already stored, or the typed reason
        /// nothing was written.
        result: ModeWriteResult,
        /// Boxed for the same reason as [`Outcome::ChatProfile`]'s read.
        read: Box<ChatConfigRead>,
    },
    /// One saved profile was APPLIED over that chat's own child, or it was not
    /// and the typed answer says why (profiles-editor T3b, S4, S8, S9): the
    /// channel's own dialog for `default` (its text carried verbatim, L30 ruling
    /// 2), the typed refusals, or nothing at all.
    ChatApply {
        chat: ChatId,
        /// The answer, with the configuration the attempt re-read when it ran.
        /// Boxed because the attempt is far larger than the other variants and
        /// every `Outcome` pays for its largest one.
        attempt: Box<ChatProfileApply>,
    },
    /// ONE fallback chain of one profile was saved over that chat's own child, or
    /// it was not and this is why (profiles-editor T5, S2, S3, L30-3). The write
    /// is followed by the same three reads a plain read makes, so the page draws
    /// the store's truth instead of what it hoped for.
    ChatChain {
        chat: ChatId,
        /// What the attempt came to: saved, already stored, a profile or chain the
        /// store does not record, a chain the owner's rules refuse, or the typed
        /// reason nothing was written.
        result: ChainWriteResult,
        /// Boxed for the same reason as [`Outcome::ChatProfile`]'s read.
        read: Box<ChatConfigRead>,
    },
}

impl ChatRuntime {
    /// Runs one command, in the folder of this process.
    pub fn execute(&self, command: ChatCommand) -> Outcome {
        self.execute_in(command, None)
    }

    /// Runs one command with the folder a workspace named for the chat it
    /// opens, worded in English: the form a caller that holds no language uses,
    /// which in this workspace is this crate's own tests. The app holds one, so
    /// it calls [`Self::execute_in_language`].
    pub fn execute_in(&self, command: ChatCommand, folder: Option<PathBuf>) -> Outcome {
        self.execute_in_language(Language::En, command, folder)
    }

    /// [`Self::execute_in`], with the app's own words in `language` in its
    /// answer. Only the two refusals this module words itself are translated:
    /// a failure the host reports travels as it arrived (D7).
    pub fn execute_in_language(
        &self,
        language: Language,
        command: ChatCommand,
        folder: Option<PathBuf>,
    ) -> Outcome {
        match command {
            ChatCommand::OpenProcess { chat } => {
                let outcome = self.open(language, chat.clone(), folder);
                // The capabilities the chat's desktop client caches (and its
                // requestId counter) belong to the child that reported them.
                // Focusing a chat whose child is already up, or on its way,
                // reaches that SAME child, so its client stays; any other
                // result (a child started now, one discarded, a failed start)
                // leaves a different child or none, so the next request asks
                // again (helper-stop S8, profiles-editor S8).
                let same_child = matches!(
                    outcome,
                    Outcome::Opened {
                        how: Opened::Existing | Opened::Starting,
                        ..
                    }
                );
                if !same_child {
                    self.forget_desktop_client(&chat);
                }
                outcome
            }
            ChatCommand::Send { chat, rpc } => self.send(language, chat, &rpc),
            // The child is stopped here, on the caller's thread: the host's grace
            // is up to 2 s and nothing of the window may wait on it (0.1.14).
            ChatCommand::CloseProcess { chat } => {
                self.close(&chat);
                Outcome::Closed { chat }
            }
            // One helper, over the desktop channel into that chat's own child
            // (helper-stop S2, S3, S6). Blocking on the channel's deadline, so
            // the caller runs it off the window's thread.
            ChatCommand::StopHelper { chat, helper_id } => {
                if self.stop_helper(&chat, &helper_id) {
                    Outcome::Stopped { chat, helper_id }
                } else {
                    Outcome::NotStopped { chat, helper_id }
                }
            }
            // The chat's own profile configuration, over the desktop channel
            // into that chat's own child (profiles-editor T2b, S9). Blocking on
            // the channel's deadline - once, because the three reads are
            // sequential - so the caller runs it off the window's thread.
            ChatCommand::ReadChatProfile { chat } => {
                let read = Box::new(self.read_chat_profile(&chat));
                Outcome::ChatProfile { chat, read }
            }
            // One profile's orchestration mode, over the desktop channel into
            // that chat's own child, and then the same read a plain read makes
            // (profiles-editor T3-mode, S2, S3). Blocking on the channel's
            // deadline under the chat's one client, so the caller runs it off
            // the window's thread.
            ChatCommand::SetChatProfileMode {
                chat,
                profile,
                mode,
            } => {
                let write = self.save_chat_profile_mode(&chat, &profile, mode);
                Outcome::ChatMode {
                    chat,
                    result: write.result,
                    read: Box::new(write.read),
                }
            }
            // One saved profile, applied over the desktop channel into that
            // chat's own child (profiles-editor T3b, S4, S8, S9): `session`
            // binds this chat only, `default` is the channel's global apply and
            // may answer the screen's own dialog first. Blocking on the
            // channel's deadline under the chat's one client, so the caller runs
            // it off the window's thread.
            ChatCommand::ApplyChatProfile {
                chat,
                profile,
                mode,
                confirmation,
            } => {
                let attempt = self.apply_chat_profile(&chat, &profile, mode, confirmation.as_ref());
                Outcome::ChatApply {
                    chat,
                    attempt: Box::new(attempt),
                }
            }
            // ONE fallback chain of one profile, over the desktop channel into
            // that chat's own child, and then the same read a plain read makes
            // (profiles-editor T5, S2, S3, L30-3). Blocking on the channel's
            // deadline under the chat's one client, so the caller runs it off the
            // window's thread.
            ChatCommand::SetChatProfileChain {
                chat,
                profile,
                agent,
                steps,
            } => {
                let write = self.save_chat_profile_chain(&chat, &profile, &agent, &steps);
                Outcome::ChatChain {
                    chat,
                    result: write.result,
                    read: Box::new(write.read),
                }
            }
            // The host owns the mark's effect (protection, reaping); the app
            // owns the mark itself, so this carries no answer to fold back.
            ChatCommand::SetWorking { chat, working } => {
                self.set_working(&chat, working);
                Outcome::Working { chat, working }
            }
        }
    }

    fn open(&self, language: Language, chat: ChatId, folder: Option<PathBuf>) -> Outcome {
        // The registry lock ends before `open`, which waits for the child.
        let key = {
            let mut keys = self.keys();
            keys.key_of(&chat).or_else(|| keys.register_new(&chat))
        };
        let Some(key) = key else {
            return Outcome::OpenFailed {
                chat,
                message: messages::chat_has_no_process_to_open(language).to_owned(),
            };
        };
        match self.host.open_in(key, folder) {
            Ok(how) => Outcome::Opened { chat, how },
            Err(error) => Outcome::OpenFailed {
                chat,
                message: error.to_string(),
            },
        }
    }

    fn send(&self, language: Language, chat: ChatId, rpc: &RpcCommand) -> Outcome {
        let key = self.keys().key_of(&chat);
        let Some(key) = key else {
            return Outcome::SendFailed {
                chat,
                message: messages::chat_has_no_process_yet(language).to_owned(),
            };
        };
        match self.host.send(&key, rpc) {
            Ok(()) => Outcome::Sent,
            Err(error) => Outcome::SendFailed {
                chat,
                message: error.to_string(),
            },
        }
    }
}
