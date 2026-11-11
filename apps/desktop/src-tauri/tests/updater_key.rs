//! Trust-anchor guard: the `plugins.updater.pubkey` embedded in
//! `tauri.conf.json` must be byte-identical to the minisign public key the
//! release key generates, recorded here in `fixtures/updater-pubkey.txt`.
//!
//! A review cannot catch a wrong character there. The value still looks like a
//! base64 minisign key, the app still reads it, still offers an update and still
//! downloads it; only the signature check refuses it, and every user sees "The
//! signature verification failed" with nothing pointing at the config. 0.1.2 and
//! 0.1.3 shipped exactly that while every other test stayed green, so the shipped
//! value is compared with the generated one instead of being trusted.

/// The generated public key, copied verbatim from `target/expected-pubkey.txt`
/// (a public key, not a secret). It is read at run time so a wrong character in
/// it is a plain test failure, never a rebuild.
const FIXTURE: &str = "tests/fixtures/updater-pubkey.txt";

/// The base64 of `untrusted comment:`, the first 18 bytes of the header every
/// minisign public key opens with (its trailing space falls inside the next
/// base64 group, so the alphabet prefix ends at this colon): a value pasted from
/// anywhere else fails here before it reaches a user.
const UNTRUSTED_COMMENT: &str = "dW50cnVzdGVkIGNvbW1lbnQ6";

/// How long the minisign public key of the release key is, in characters.
const KEY_LEN: usize = 152;

#[test]
fn the_embedded_updater_key_is_the_generated_public_key() {
    let config = include_str!("../tauri.conf.json");
    let embedded = embedded_pubkey(config);

    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let expected = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("{} is missing or unreadable: {e}", fixture_path.display()));

    assert_eq!(
        embedded,
        expected.trim_end_matches(['\r', '\n']),
        "the embedded updater key no longer matches the generated public key; \
         a wrong character makes every update fail verification"
    );

    assert_eq!(
        embedded.chars().count(),
        KEY_LEN,
        "the updater key must be the {KEY_LEN}-character minisign public key, \
         got {} characters",
        embedded.chars().count()
    );
    assert!(
        embedded.is_ascii(),
        "the updater key must be ASCII, so no non-ASCII look-alike slips in"
    );
    assert!(
        !embedded.chars().any(char::is_whitespace),
        "the updater key must be one unbroken value, without whitespace"
    );
    assert!(
        embedded
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='),
        "the updater key must use only the base64 alphabet, got {embedded:?}"
    );
    assert!(
        embedded.starts_with(UNTRUSTED_COMMENT),
        "the updater key must start with the base64 of \"untrusted comment:\", \
         which every minisign public key does, got {embedded:?}"
    );
}

/// The `"pubkey"` string value of the updater block: the key, then the `:` and
/// the quoted value after it. Plain string handling, so no JSON crate is needed
/// to read one string out of one file.
fn embedded_pubkey(config: &str) -> &str {
    let key = "\"pubkey\"";
    let key_at = config
        .find(key)
        .expect("tauri.conf.json has no \"pubkey\" key: the updater's trust anchor is gone");
    let after_key = &config[key_at + key.len()..];
    let colon_at = after_key
        .find(':')
        .expect("\"pubkey\" is not followed by a ':'");
    let after_colon = &after_key[colon_at + 1..];
    let open_at = after_colon
        .find('"')
        .expect("\"pubkey\" has no quoted value");
    let quoted = &after_colon[open_at + 1..];
    let close_at = quoted.find('"').expect("\"pubkey\" has no closing quote");
    &quoted[..close_at]
}
