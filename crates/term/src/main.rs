//! `gentle-term`: send prompts to the real `gentle-shell --mode rpc` and
//! stream the replies. Usage:
//!
//! ```text
//! gentle-term [--isolated] [--provider P] [--model M] [prompt...]
//! ```
//!
//! With no prompt, every line read from stdin is a prompt. Exit code 1 if the
//! model failed (credits, rate limit...) or the child died.
//!
//! All state and decisions live in `gentle-app`; this file only runs its
//! commands against the backend and prints what it renders.

use std::io::{self, BufRead, Write};
use std::process::ExitCode;
use std::time::Duration;

use gentle_app::{update, AppState, Command, Input, Intent, NoticeLevel};
use gentle_backend::launcher::{self, LauncherEnv};
use gentle_backend::process::{ProcessEvent, RpcProcess, SpawnOptions};
use gentle_core::chat::Dialog;
use gentle_core::rpc::{Decoded, RpcEvent, UiAnswer};
use gentle_term::render::{Out, Renderer};

fn main() -> ExitCode {
    let (forwarded, prompt) = split_args(std::env::args().skip(1).collect());
    match run(forwarded, prompt) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(message) => {
            eprintln!("gentle-term: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Flags forwarded to gentle-shell, and the prompt words after them.
fn split_args(args: Vec<String>) -> (Vec<String>, String) {
    let mut forwarded = Vec::new();
    let mut words = Vec::new();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--isolated" => forwarded.push(arg),
            "--provider" | "--model" => {
                forwarded.push(arg);
                forwarded.extend(iter.next());
            }
            _ => words.push(arg),
        }
    }
    (forwarded, words.join(" "))
}

/// The session: app state, what was already printed, and the child.
struct Session {
    process: RpcProcess,
    state: AppState,
    renderer: Renderer,
    printed_notices: usize,
    model_error: bool,
}

impl Session {
    /// Feeds one input to the app layer, runs its commands, prints the result.
    fn dispatch(&mut self, input: Input) -> Result<(), String> {
        let (state, commands) = update(std::mem::take(&mut self.state), input);
        self.state = state;
        for command in commands {
            match command {
                Command::Send(rpc) => self
                    .process
                    .send(&rpc)
                    .map_err(|e| format!("cannot reach gentle-shell: {e}"))?,
                Command::CopyToClipboard(_) => {} // no clipboard in a terminal
            }
        }
        self.print();
        Ok(())
    }

    fn print(&mut self) {
        let new_notices = self.state.notices.iter().skip(self.printed_notices);
        for notice in new_notices {
            let level = match notice.level {
                NoticeLevel::Info => "info",
                NoticeLevel::Warning => "warning",
                NoticeLevel::Error => "error",
            };
            print_out(&Out::Notice {
                level: level.into(),
                text: notice.text.clone(),
            });
        }
        self.printed_notices = self.state.notices.len();
        for out in self.renderer.render(&self.state.chat) {
            self.model_error |= matches!(out, Out::ModelError(_));
            print_out(&out);
        }
    }

    /// Nobody can answer extension dialogs here: cancel so the agent never hangs.
    fn cancel_dialogs(&mut self) -> Result<(), String> {
        let ids: Vec<String> = self
            .state
            .chat
            .pending_dialogs
            .iter()
            .map(|dialog| dialog_id(dialog).to_owned())
            .collect();
        for id in ids {
            eprintln!("[gentle-term] cancelled extension dialog: {id}");
            self.dispatch(Input::Intent(Intent::AnswerDialog {
                id,
                answer: UiAnswer::Cancelled,
            }))?;
        }
        Ok(())
    }
}

/// Runs the session; `Ok(true)` when every turn finished without a model error.
fn run(forwarded: Vec<String>, prompt: String) -> Result<bool, String> {
    let launcher = launcher::resolve(&LauncherEnv::from_process()).map_err(|e| e.to_string())?;
    let options = SpawnOptions {
        args: forwarded.into_iter().map(Into::into).collect(),
        // pi's rpc mode only serves extension UI requests with an interactive host.
        env: vec![("GENTLE_SHELL_INTERACTIVE_HOST".into(), "1".into())],
        cwd: None,
    };
    let process = RpcProcess::spawn(&launcher, &options)
        .map_err(|e| format!("cannot start {}: {e}", launcher.program.display()))?;
    let mut session = Session {
        process,
        state: AppState::default(),
        renderer: Renderer::new(),
        printed_notices: 0,
        model_error: false,
    };

    let prompts: Box<dyn Iterator<Item = String>> = if prompt.is_empty() {
        Box::new(io::stdin().lock().lines().map_while(Result::ok))
    } else {
        Box::new(std::iter::once(prompt))
    };
    for message in prompts.filter(|line| !line.trim().is_empty()) {
        session.dispatch(Input::Intent(Intent::SendPrompt(message)))?;
        if !run_turn(&mut session)? {
            return Err("gentle-shell exited unexpectedly".into());
        }
    }
    let ok = !session.model_error;
    let _ = session.process.shutdown(Duration::from_secs(5));
    Ok(ok)
}

/// Consumes events until the agent settles; `false` if the child went away.
fn run_turn(session: &mut Session) -> Result<bool, String> {
    let mut started = false;
    loop {
        let Ok(event) = session.process.events().recv() else {
            return Ok(false);
        };
        let decoded = match event {
            ProcessEvent::Closed => return Ok(false),
            ProcessEvent::Stderr(line) => {
                eprintln!("[gentle-shell] {line}");
                continue;
            }
            ProcessEvent::Output(decoded) => decoded,
        };
        let Decoded::Event(event) = decoded else {
            continue;
        };
        if let RpcEvent::Response(response) = &event {
            if let Err(error) = &response.outcome {
                return Err(format!("{} rejected: {error}", response.command));
            }
        }
        let turn_over = match &event {
            RpcEvent::AgentSettled => true,
            RpcEvent::AgentEnd { will_retry } => *will_retry != Some(true),
            _ => false,
        };
        session.dispatch(Input::Rpc(event))?;
        session.cancel_dialogs()?;
        started |= session.state.chat.working;
        if started && turn_over {
            return Ok(true);
        }
    }
}

fn dialog_id(dialog: &Dialog) -> &str {
    match dialog {
        Dialog::Select { id, .. }
        | Dialog::Confirm { id, .. }
        | Dialog::Input { id, .. }
        | Dialog::Editor { id, .. } => id,
    }
}

fn print_out(out: &Out) {
    match out {
        Out::Text(text) => {
            print!("{text}");
            let _ = io::stdout().flush();
        }
        Out::EndOfReply => println!(),
        Out::ModelError(error) => eprintln!("\n✗ model error: {error}"),
        Out::Notice { level, text } => eprintln!("[{level}] {text}"),
    }
}
