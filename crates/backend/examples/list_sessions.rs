//! Lists the saved sessions of a pi home and shows what each refresh cost.
//!
//! `cargo run -p gentle-backend --example list_sessions -- <home>`
//! (e.g. `C:\Users\<you>\.gentle-shell\agent`).

use std::time::Instant;

use gentle_backend::session_index::SessionIndex;

fn main() {
    let home = std::env::args_os()
        .nth(1)
        .expect("usage: list_sessions <home>");
    let mut index = SessionIndex::new(std::path::Path::new(&home).join("sessions"));
    for pass in ["first refresh", "second refresh"] {
        let started = Instant::now();
        let stats = index.refresh().expect("refresh");
        println!("{pass}: {stats:?} in {:?}", started.elapsed());
    }
    for entry in index.entries() {
        let s = &entry.summary;
        let title = s
            .name
            .as_deref()
            .or(s.first_message.as_deref())
            .unwrap_or("(no messages)");
        let title: String = title.chars().take(60).collect();
        println!(
            "{:>13}  {:>3} msgs  {}  {}",
            entry.modified_ms,
            s.message_count,
            s.cwd,
            title.replace('\n', " ")
        );
    }
}
