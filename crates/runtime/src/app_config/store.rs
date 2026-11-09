//! Reading and writing the config file.
//!
//! A missing file is the empty config and no notice. A file this app does not
//! understand is never overwritten: it is renamed aside
//! (`config.json.corrupt-<unix seconds>`, and never over an earlier aside file),
//! the app starts with an empty config, and the reason comes back as a
//! [`ConfigNotice`]. A notice is data: how it is shown is the caller's business.
//!
//! The write goes to a temporary file in the same directory and is renamed over
//! the target, so a reader never sees a half-written file. No lock is taken
//! anywhere here, and no I/O happens under one.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::config::{AppConfig, CONFIG_VERSION};

mod aside;
mod document;

/// The config to go on with, and what the user should be told about the file it
/// came from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Loaded {
    pub config: AppConfig,
    /// Why the file could not be used, when it could not.
    pub notice: Option<ConfigNotice>,
}

/// What was wrong with the file, as the user-facing reason is built from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigProblem {
    /// Not the document this app writes: unreadable JSON, or a shape that does
    /// not parse. Without a version to read, it is taken as this.
    Damaged,
    /// Valid JSON naming a version this app does not write.
    UnknownVersion { found: u64 },
}

/// The notice for an unusable file, and what became of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigNotice {
    pub problem: ConfigProblem,
    /// Where the unusable file was kept, `None` when it could not be moved and
    /// is still where it was.
    pub aside: Option<PathBuf>,
}

impl ConfigNotice {
    /// The reason, as one line of plain text for any surface to show as it
    /// likes. Carries no markup and no locale.
    pub fn message(&self) -> String {
        let kept = match &self.aside {
            Some(aside) => format!("It was kept at {}", aside.display()),
            None => "It was left where it was".to_owned(),
        };
        match self.problem {
            ConfigProblem::Damaged => format!(
                "The app config file could not be read. {kept}, and gentle-desk \
                 started with an empty config."
            ),
            ConfigProblem::UnknownVersion { found } => format!(
                "The app config file is version {found}, which this version of \
                 gentle-desk does not write (it writes version {CONFIG_VERSION}). \
                 {kept}, and gentle-desk started with an empty config."
            ),
        }
    }
}

/// The config at `path`, or the empty config when there is no file.
pub fn load(path: &Path) -> Loaded {
    let data = match fs::read(path) {
        Ok(data) => data,
        // No file is the normal first run, not an error.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Loaded::default(),
        // There is a file but it cannot be read: it is left exactly as it is.
        Err(_) => {
            return Loaded {
                config: AppConfig::default(),
                notice: Some(ConfigNotice {
                    problem: ConfigProblem::Damaged,
                    aside: None,
                }),
            }
        }
    };
    match document::parse(&data) {
        Ok(config) => Loaded {
            config,
            notice: None,
        },
        // Never overwritten: moved aside, and the reason goes back to the
        // caller to show as it likes.
        Err(problem) => Loaded {
            config: AppConfig::default(),
            notice: Some(ConfigNotice {
                problem,
                aside: aside::move_aside(path).ok(),
            }),
        },
    }
}

/// Writes the config to `path`, creating its directory, through a temporary
/// file next to it. On any failure the previous file is left as it was and the
/// temporary file is removed.
pub fn save(path: &Path, config: &AppConfig) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!("{}.tmp", file_name(path)));
    if let Err(error) = write(&temp, config) {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    fs::rename(&temp, path).inspect_err(|_| {
        let _ = fs::remove_file(&temp);
    })
}

/// Writes the bytes of `config` to `path`.
fn write(path: &Path, config: &AppConfig) -> io::Result<()> {
    let data = document::bytes(config)?;
    let mut file = fs::File::create(path)?;
    file.write_all(&data)
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || "config".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
mod tests;
