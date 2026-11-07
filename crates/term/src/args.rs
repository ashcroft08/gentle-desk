//! Command-line parsing: what goes to gentle-shell, and what is the prompt.
//!
//! Pure, no I/O: `main` hands it `std::env::args().skip(1)` and gets back the
//! flags to forward to the child plus the prompt text.

/// Flags forwarded to gentle-shell, and the prompt words after them.
///
/// `--isolated` stands alone; `--provider` and `--model` take the next argument
/// as their value. Anything else is a prompt word, joined with single spaces.
/// A value-taking flag whose value is missing is forwarded alone (gentle-shell
/// then rejects it) so the prompt stays empty.
pub fn split_args(args: Vec<String>) -> (Vec<String>, String) {
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

#[cfg(test)]
mod tests;
