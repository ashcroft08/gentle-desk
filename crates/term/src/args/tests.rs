use super::*;

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|word| (*word).to_owned()).collect()
}

#[test]
fn no_args_means_no_flags_and_no_prompt() {
    assert_eq!(split_args(words(&[])), (Vec::new(), String::new()));
}

#[test]
fn flags_are_forwarded_and_the_words_after_them_are_the_prompt() {
    let args = [
        "--isolated",
        "--provider",
        "openai",
        "--model",
        "gpt-5",
        "hola",
        "mundo",
    ];
    assert_eq!(
        split_args(words(&args)),
        (
            words(&["--isolated", "--provider", "openai", "--model", "gpt-5"]),
            "hola mundo".to_owned()
        )
    );
}

#[test]
fn a_flag_whose_value_is_missing_is_forwarded_alone() {
    assert_eq!(
        split_args(words(&["--provider"])),
        (words(&["--provider"]), String::new())
    );
}

#[test]
fn a_flag_between_prompt_words_does_not_split_the_prompt() {
    let args = ["hola", "--model", "gpt-5", "mundo"];
    assert_eq!(
        split_args(words(&args)),
        (words(&["--model", "gpt-5"]), "hola mundo".to_owned())
    );
}

#[test]
fn prompt_words_alone_are_joined_with_single_spaces() {
    assert_eq!(
        split_args(words(&["hola", "que", "tal"])),
        (Vec::new(), "hola que tal".to_owned())
    );
}
