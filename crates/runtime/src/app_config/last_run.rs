//! The version of the app the last run of it was.
//!
//! An ADDITIVE field of version 1, read LENIENTLY like the theme and the window
//! state: absent, or a value that is not text, is "no version stored" - the first
//! run of a build that predates this feature - never a damaged file, so the
//! workspaces under it survive and the document version does not move.

use serde::{Deserialize, Deserializer};

/// The stored version, or `None` for an absent, unknown-shaped or mistyped value,
/// so the rest of the document (the theme, the workspaces) survives what this
/// field holds. Only text is a version: a number, a flag or an object reads as
/// nothing stored.
pub(crate) fn or_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let stored = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(stored.and_then(|value| value.as_str().map(str::to_owned)))
}

#[cfg(test)]
mod tests {
    use crate::app_config::{AppConfig, Theme, CONFIG_VERSION};

    #[test]
    fn a_default_config_stores_no_last_run_version() {
        assert_eq!(AppConfig::default().last_run_version, None);
    }

    #[test]
    fn a_config_without_the_field_loads_with_nothing_stored() {
        // The file every release before this feature wrote.
        let data =
            br#"{"version":1,"theme":"dark","workspaces":[{"path":"C:/dev/p","collapsed":false}]}"#;
        let config: AppConfig = serde_json::from_slice(data).expect("a v1 document loads");
        assert_eq!(config.last_run_version, None);
        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.theme, Theme::Dark);
        assert_eq!(config.workspaces[0].path, "C:/dev/p");
    }

    #[test]
    fn an_unknown_shaped_last_run_version_stores_nothing_and_keeps_the_rest() {
        let values: [&[u8]; 4] = [br#"42"#, br#"null"#, br#"{"version":"0.1.8"}"#, br#"true"#];
        for value in values {
            let data = format!(
                r#"{{"version":1,"theme":"Gentle","last_run_version":{},"workspaces":[{{"path":"C:/dev/p","collapsed":true}}]}}"#,
                std::str::from_utf8(value).expect("test input is text")
            );
            let config: AppConfig = serde_json::from_str(&data)
                .unwrap_or_else(|error| panic!("{data} is not a damaged file: {error}"));
            assert_eq!(config.last_run_version, None, "{data} stores nothing");
            assert_eq!(config.theme, Theme::Gentle, "the theme survives {data}");
            assert!(
                config.workspaces[0].collapsed,
                "the workspaces survive {data}"
            );
        }
    }

    #[test]
    fn a_stored_version_round_trips_through_the_document() {
        let mut config = AppConfig::default();
        assert!(config.set_last_run_version("0.1.8"));
        let bytes = serde_json::to_vec(&config).expect("the document writes");
        let back: AppConfig = serde_json::from_slice(&bytes).expect("and reads back");
        assert_eq!(back.last_run_version, Some("0.1.8".to_owned()));
    }

    #[test]
    fn storing_the_same_version_again_changes_nothing() {
        let mut config = AppConfig::default();
        assert!(config.set_last_run_version("0.1.8"));
        assert!(!config.set_last_run_version("0.1.8"));
        assert!(config.set_last_run_version("0.1.9"));
        assert_eq!(config.last_run_version, Some("0.1.9".to_owned()));
    }

    #[test]
    fn a_document_with_nothing_stored_writes_no_field() {
        // A file saved by a run that never stored a version stays the document
        // the older builds wrote, exactly as the window state does.
        let config = AppConfig::default();
        let bytes = serde_json::to_vec(&config).expect("the document writes");
        let text = String::from_utf8(bytes).expect("the document is text");
        assert!(
            !text.contains("last_run_version"),
            "nothing stored writes no field: {text}"
        );
    }
}
