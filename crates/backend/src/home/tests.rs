use super::*;

fn os(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

fn env() -> HomeEnv {
    HomeEnv {
        user_home: Some(PathBuf::from("/u")),
        ..HomeEnv::default()
    }
}

#[test]
fn with_no_home_flag_the_home_is_isolated() {
    assert_eq!(
        with_home_flag(os(&["--model", "m"]), &env()),
        os(&["--model", "m", "--isolated"])
    );
}

#[test]
fn an_explicit_home_flag_is_kept_untouched() {
    for args in [
        &["--isolated"][..],
        &["--link"],
        &["--home", "/x"],
        &["--model", "m", "--link"],
    ] {
        assert_eq!(with_home_flag(os(args), &env()), os(args));
    }
}

#[test]
fn gentle_shell_home_becomes_a_home_flag_when_no_flag_is_given() {
    let env = HomeEnv {
        gentle_shell_home: Some(PathBuf::from("/custom")),
        ..env()
    };
    assert_eq!(with_home_flag(os(&[]), &env), os(&["--home", "/custom"]));
    // An explicit flag still wins.
    assert_eq!(with_home_flag(os(&["--link"]), &env), os(&["--link"]));
}

#[test]
fn isolated_lives_under_gentle_shell_agent() {
    assert_eq!(
        agent_dir(&os(&["--isolated"]), &env()),
        Some(PathBuf::from("/u/.gentle-shell/agent"))
    );
}

#[test]
fn no_flag_means_isolated() {
    assert_eq!(
        agent_dir(&os(&["--model", "m"]), &env()),
        Some(PathBuf::from("/u/.gentle-shell/agent"))
    );
}

#[test]
fn link_uses_pi_coding_agent_dir_else_dot_pi_agent() {
    assert_eq!(
        agent_dir(&os(&["--link"]), &env()),
        Some(PathBuf::from("/u/.pi/agent"))
    );
    let env = HomeEnv {
        pi_coding_agent_dir: Some(PathBuf::from("/pi")),
        ..env()
    };
    assert_eq!(
        agent_dir(&os(&["--link"]), &env),
        Some(PathBuf::from("/pi"))
    );
}

#[test]
fn an_explicit_home_wins_over_everything() {
    let env = HomeEnv {
        gentle_shell_home: Some(PathBuf::from("/other")),
        ..env()
    };
    assert_eq!(
        agent_dir(&os(&["--home", "/x", "--isolated"]), &env),
        Some(PathBuf::from("/x"))
    );
}

#[test]
fn gentle_shell_home_applies_when_no_flag_is_given() {
    let env = HomeEnv {
        gentle_shell_home: Some(PathBuf::from("/custom")),
        ..env()
    };
    assert_eq!(agent_dir(&os(&[]), &env), Some(PathBuf::from("/custom")));
}

#[test]
fn without_a_user_home_isolated_cannot_be_resolved() {
    assert_eq!(agent_dir(&os(&["--isolated"]), &HomeEnv::default()), None);
}

#[test]
fn a_dangling_home_flag_resolves_to_nothing_useful() {
    assert_eq!(agent_dir(&os(&["--home"]), &env()), None);
}

#[test]
fn a_chosen_link_adds_the_link_flag() {
    let env = HomeEnv {
        choice: Some(HomeChoice::Link),
        ..env()
    };

    assert_eq!(
        with_home_flag(os(&["--model", "m"]), &env),
        os(&["--model", "m", "--link"])
    );
}

#[test]
fn a_chosen_separate_home_adds_the_isolated_flag() {
    let env = HomeEnv {
        choice: Some(HomeChoice::Isolated),
        ..env()
    };

    assert_eq!(with_home_flag(os(&[]), &env), os(&["--isolated"]));
}

#[test]
fn the_developer_home_variable_beats_the_choice() {
    let env = HomeEnv {
        choice: Some(HomeChoice::Link),
        gentle_shell_home: Some(PathBuf::from("/custom")),
        ..env()
    };

    assert_eq!(with_home_flag(os(&[]), &env), os(&["--home", "/custom"]));
}

#[test]
fn an_explicit_flag_beats_the_choice() {
    let env = HomeEnv {
        choice: Some(HomeChoice::Link),
        ..env()
    };

    assert_eq!(
        with_home_flag(os(&["--isolated"]), &env),
        os(&["--isolated"])
    );
}

#[test]
fn a_chosen_link_reads_the_sessions_of_pi() {
    let env = HomeEnv {
        choice: Some(HomeChoice::Link),
        ..env()
    };
    let args = with_home_flag(os(&[]), &env);

    assert_eq!(agent_dir(&args, &env), Some(PathBuf::from("/u/.pi/agent")));
}

#[test]
fn pi_is_found_where_the_default_or_the_variable_says_and_the_folder_exists() {
    let default = PathBuf::from("/u/.pi/agent");
    assert_eq!(
        detect_pi(&env(), |path| path == default),
        Some(default.clone())
    );
    assert_eq!(detect_pi(&env(), |_| false), None);

    let custom = HomeEnv {
        pi_coding_agent_dir: Some(PathBuf::from("/elsewhere")),
        ..env()
    };
    assert_eq!(
        detect_pi(&custom, |path| path == Path::new("/elsewhere")),
        Some(PathBuf::from("/elsewhere"))
    );
    assert_eq!(detect_pi(&custom, |path| path == default), None);
}

#[test]
fn a_choice_is_spelled_the_way_the_config_stores_it() {
    assert_eq!(
        serde_json::to_string(&HomeChoice::Link).unwrap(),
        "\"link\""
    );
    assert_eq!(
        serde_json::to_string(&HomeChoice::Isolated).unwrap(),
        "\"isolated\""
    );
    assert_eq!(HomeChoice::from_name("link"), Some(HomeChoice::Link));
    assert_eq!(HomeChoice::from_name("separate"), None);
}
