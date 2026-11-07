//! The disk side of the session index: which files exist and how to fold the
//! bytes past an offset into a scanner.

use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use gentle_core::rpc::LineSplitter;
use gentle_core::session::SessionScanner;

/// `<dir>/*/*.jsonl`: pi keeps one subdirectory per project cwd.
pub fn session_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let projects = match fs::read_dir(dir) {
        Ok(projects) => projects,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let mut files = Vec::new();
    for project in projects.flatten() {
        if !project.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }
        let Ok(inside) = fs::read_dir(project.path()) else {
            continue;
        };
        files.extend(
            inside
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "jsonl")),
        );
    }
    files.sort();
    Ok(files)
}

/// Feeds every complete line from byte `offset` on into `scanner`. Returns the
/// new parsed length (just past the last `\n`) and the bytes read. A trailing
/// partial line (a write in progress) stays unparsed for the next refresh.
pub fn read_new_lines(
    path: &Path,
    offset: u64,
    scanner: &mut SessionScanner,
) -> io::Result<(u64, u64)> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let read = data.len() as u64;
    let Some(last_newline) = data.iter().rposition(|&b| b == b'\n') else {
        return Ok((offset, read));
    };
    let complete = &data[..=last_newline];
    let mut splitter = LineSplitter::new();
    for line in splitter.push(complete) {
        scanner.push_line(&line);
    }
    Ok((offset + complete.len() as u64, read))
}
