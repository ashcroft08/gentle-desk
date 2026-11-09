//! The plain open of the runtime names no folder: the app layer passes folders
//! in a later slice, and until then every child inherits the process folder.

use std::path::PathBuf;

use super::*;

#[test]
fn the_plain_open_names_no_folder() {
    let (runtime, spawner) = runtime();
    let chat = ChatId::of_session("s.jsonl");

    runtime.execute(open(&chat));

    assert_eq!(spawner.folders(), vec![None::<PathBuf>]);
}
