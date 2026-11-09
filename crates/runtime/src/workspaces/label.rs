//! The labels of the workspace rows: the name, extended with the parent
//! segments it takes to tell two workspaces with the same name apart.

use std::collections::HashMap;

/// The label of each folder, in the order the folders were gathered.
pub(super) fn labels(names: &[String], parts: &[Vec<&str>]) -> Vec<String> {
    let mut labels: Vec<String> = vec![String::new(); names.len()];
    let mut groups: HashMap<&str, Vec<usize>> = HashMap::new();
    for (at, name) in names.iter().enumerate() {
        groups.entry(name.as_str()).or_default().push(at);
    }
    for (name, members) in groups {
        match members.as_slice() {
            // The only workspace of its name reads as just its name.
            [only] => labels[*only] = name.to_owned(),
            _ => extend(&members, names, parts, &mut labels),
        }
    }
    labels
}

/// Gives each member of one name group the shortest end of its path that no
/// other member of that group shares: one more parent segment per round, only
/// for the members still alike, so a label is never longer than it has to be.
/// The group is finite and its paths differ, so the rounds end.
fn extend(members: &[usize], names: &[String], parts: &[Vec<&str>], labels: &mut [String]) {
    let mut pending: Vec<usize> = members.to_vec();
    let mut depth = 1;
    while !pending.is_empty() {
        let suffixes: Vec<String> = pending
            .iter()
            .map(|&at| suffix(at, names, parts, depth))
            .collect();
        let mut seen: HashMap<&str, usize> = HashMap::new();
        for suffix in &suffixes {
            *seen.entry(suffix.as_str()).or_default() += 1;
        }
        let mut still: Vec<usize> = Vec::new();
        for (&at, suffix) in pending.iter().zip(&suffixes) {
            // A member whose suffix is its whole path cannot grow any further;
            // two workspaces differ by key, so this only bounds the loop.
            if seen[suffix.as_str()] == 1 || parts[at].len() <= depth {
                labels[at] = suffix.clone();
            } else {
                still.push(at);
            }
        }
        pending = still;
        depth += 1;
    }
}

/// What the label of `at` shows at `depth`: its name at depth one (which is the
/// whole path for a root), then the end of its path that many segments long,
/// which is never more than the path has.
fn suffix(at: usize, names: &[String], parts: &[Vec<&str>], depth: usize) -> String {
    if depth <= 1 {
        return names[at].clone();
    }
    let parts = &parts[at];
    parts[parts.len().saturating_sub(depth)..].join("/")
}
