//! Folding the child's output into the app: RPC events, its stderr, and the
//! process going away.

use gentle_app::Input;
use gentle_backend::process::ProcessEvent;
use gentle_core::rpc::Decoded;
use iced::Task;

use super::App;
use crate::clock;
use crate::message::Message;

impl App {
    pub(super) fn on_process(&mut self, event: ProcessEvent) -> Task<Message> {
        match event {
            ProcessEvent::Output(Decoded::Event(event)) => {
                self.dispatch(Input::RpcAt(event, clock::now_ms()))
            }
            ProcessEvent::Output(Decoded::Unknown(_)) => Task::none(),
            ProcessEvent::Stderr(line) => {
                eprintln!("[gentle-shell] {line}");
                Task::none()
            }
            ProcessEvent::Closed => {
                self.fatal
                    .get_or_insert_with(|| "gentle-shell exited".into());
                Task::none()
            }
        }
    }
}
