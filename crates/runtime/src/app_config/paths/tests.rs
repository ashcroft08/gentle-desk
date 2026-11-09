use super::*;
use std::ffi::OsStr;
use std::path::PathBuf;

#[test]
fn the_config_file_sits_under_the_roaming_app_data_directory() {
    let path = config_path(Some(OsStr::new(r"C:\Users\ana\AppData\Roaming"))).unwrap();
    assert_eq!(
        path,
        PathBuf::from(r"C:\Users\ana\AppData\Roaming")
            .join("gentle-desk")
            .join("config.json")
    );
}

#[test]
fn the_config_file_is_not_the_session_index_cache() {
    let app_data = OsStr::new(r"C:\Users\ana\AppData\Roaming");
    let local_app_data = OsStr::new(r"C:\Users\ana\AppData\Local");
    let config = config_path(Some(app_data)).unwrap();
    let cache = crate::cache_path::cache_file(Some(local_app_data.to_os_string())).unwrap();
    assert_ne!(config, cache);
    assert_ne!(
        config.file_name().unwrap(),
        cache.file_name().unwrap(),
        "the config must not share the cache file name"
    );
}

#[test]
fn no_app_data_is_no_config_file() {
    assert_eq!(config_path(None), None);
}

#[test]
fn an_empty_app_data_is_no_config_file() {
    assert_eq!(config_path(Some(OsStr::new(""))), None);
}
