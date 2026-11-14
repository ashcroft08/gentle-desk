//! The languages gentle-desk speaks about its own words.
//!
//! Only the app's own UI text and the notices, errors and validations this
//! workspace generates are translated. Everything that comes from the shell or
//! from pi is shown exactly as it arrives, so nothing here touches it.

mod language;
pub mod messages;

pub use language::Language;
