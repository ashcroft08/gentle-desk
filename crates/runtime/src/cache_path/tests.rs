use super::*;

#[test]
fn the_cache_file_sits_under_the_local_app_data_directory() {
    let path = cache_file(Some(OsString::from("/home/ana/.local/share"))).unwrap();
    assert_eq!(
        path,
        PathBuf::from("/home/ana/.local/share")
            .join("gentle-desk")
            .join("session-index.json")
    );
}

#[test]
fn no_local_app_data_is_no_cache_file() {
    assert_eq!(cache_file(None), None);
}

#[test]
fn an_empty_local_app_data_is_no_cache_file() {
    assert_eq!(cache_file(Some(OsString::new())), None);
}
