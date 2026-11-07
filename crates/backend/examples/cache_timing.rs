//! Measures what the persisted session index cache saves at startup.
//!
//! Usage, from the repo root (release, as measured in the task notes):
//!
//! ```text
//! cargo run --release -p gentle-backend --example cache_timing -- <sessions dir> <cache file>
//! ```
//!
//! `cold` reads every session file. The run then writes the cache and builds a
//! second, fresh index from it: `warm` should only `stat` the files and still
//! list exactly the same sessions.

use std::path::PathBuf;
use std::time::Instant;

use gentle_backend::session_cache;
use gentle_backend::session_index::SessionIndex;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let (Some(sessions), Some(cache)) = (args.next(), args.next()) else {
        eprintln!("usage: cache_timing <sessions dir> <cache file>");
        std::process::exit(2);
    };
    let (sessions, cache) = (PathBuf::from(sessions), PathBuf::from(cache));

    let mut cold = SessionIndex::new(&sessions);
    let start = Instant::now();
    let stats = cold.refresh().expect("cannot read the sessions directory");
    println!("cold:  {:>10.1?} {:?}", start.elapsed(), stats);
    let cold_items = cold.entries();

    session_cache::save(&cache, &cold.snapshot()).expect("cannot write the cache");
    let bytes = std::fs::metadata(&cache).map_or(0, |m| m.len());
    println!("cache: {:>10} bytes at {}", bytes, cache.display());

    let mut warm = SessionIndex::new(&sessions);
    if let Some(snapshot) = session_cache::load(&cache) {
        warm.restore(snapshot);
    }
    let start = Instant::now();
    let stats = warm.refresh().expect("cannot read the sessions directory");
    println!("warm:  {:>10.1?} {:?}", start.elapsed(), stats);
    let warm_items = warm.entries();

    println!(
        "sessions: {} cold, {} warm",
        cold_items.len(),
        warm_items.len()
    );
    println!("same list: {}", cold_items == warm_items);
}
