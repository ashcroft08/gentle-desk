//! `gentle-term`: send prompts to the real `gentle-shell --mode rpc` and
//! stream the replies. Usage:
//!
//! ```text
//! gentle-term [--isolated] [--provider P] [--model M] [prompt...]
//! ```
//!
//! With no prompt, every line read from stdin is a prompt. Exit code 1 if the
//! model failed (credits, rate limit...) or the child died.

use std::io::{self, BufRead, Write};
use std::process::ExitCode;
use std::time::Duration;

use gentle_backend::launcher::{self, LauncherEnv};
use gentle_backend::process::{ProcessEvent, RpcProcess, SpawnOptions};
use gentle_core::chat::{reduce_chat, ChatState, Dialog};
use gentle_core::rpc::{Decoded, RpcCommand, RpcEvent, UiAnswer};
use gentle_term::render::{notice, Out, Renderer};

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

/// Runs the session; `Ok(true)` when every turn finished without a model error.
fn run(forwarded: Vec<String>, prompt: String) -> Result<bool, String> {
    let launcher = launcher::resolve(&LauncherEnv::from_process()).map_err(|e| e.to_string())?;
    let options = SpawnOptions {
        args: forwarded.into_iter().map(Into::into).collect(),
        // pi's rpc mode only serves extension UI requests with an interactive host.
        env: vec![("GENTLE_SHELL_INTERACTIVE_HOST".into(), "1".into())],
        cwd: None,
    };
    let mut process = RpcProcess::spawn(&launcher, &options)
        .map_err(|e| format!("cannot start {}: {e}", launcher.program.display()))?;

    let mut all_ok = true;
    let mut state = ChatState::default();
    let mut renderer = Renderer::new();
    let prompts: Box<dyn Iterator<Item = String>> = if prompt.is_empty() {
        Box::new(io::stdin().lock().lines().map_while(Result::ok))
    } else {
        Box::new(std::iter::once(prompt))
    };
    for message in prompts.filter(|line| !line.trim().is_empty()) {
        process
            .send(&RpcCommand::Prompt { message })
            .map_err(|e| format!("cannot reach gentle-shell: {e}"))?;
        match run_turn(&mut process, &mut state, &mut renderer)? {
            TurnEnd::Settled { model_error } => all_ok &= !model_error,
            TurnEnd::ChildGone => return Err("gentle-shell exited unexpectedly".into()),
        }
    }
    let _ = process.shutdown(Duration::from_secs(5));
    Ok(all_ok)
}

enum TurnEnd {
    Settled { model_error: bool },
    ChildGone,
}

/// Consumes events until the agent settles after having started working.
fn run_turn(
    process: &mut RpcProcess,
    state: &mut ChatState,
    renderer: &mut Renderer,
) -> Result<TurnEnd, String> {
    let mut model_error = false;
    let mut started = false;
    let mut answered = 0;
    loop {
        let Ok(event) = process.events().recv() else {
            return Ok(TurnEnd::ChildGone);
        };
        let decoded = match event {
            ProcessEvent::Closed => return Ok(TurnEnd::ChildGone),
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

        *state = reduce_chat(std::mem::take(state), &event);
        let outs = notice(&event).into_iter().chain(renderer.render(state));
        for out in outs {
            model_error |= matches!(out, Out::ModelError(_));
            print_out(&out);
        }

        // Nobody can answer extension dialogs here: cancel so the agent never hangs.
        for dialog in state.pending_dialogs.iter().skip(answered) {
            let _ = process.send(&RpcCommand::UiResponse {
                id: dialog_id(dialog).to_owned(),
                answer: UiAnswer::Cancelled,
            });
            eprintln!(
                "[gentle-term] cancelled extension dialog: {}",
                dialog_id(dialog)
            );
        }
        answered = state.pending_dialogs.len();

        let turn_over = match &event {
            RpcEvent::AgentSettled => true,
            RpcEvent::AgentEnd { will_retry } => *will_retry != Some(true),
            _ => false,
        };
        started |= state.working;
        if started && turn_over {
            return Ok(TurnEnd::Settled { model_error });
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
