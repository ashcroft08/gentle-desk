//! Cryptographic trust-anchor guard: the updater public key embedded in
//! `tauri.conf.json` must actually verify a signature made by the release
//! private key, checked with the same minisign verifier `tauri-plugin-updater`
//! runs at update time.
//!
//! `updater_key.rs` compares text: the embedded base64 with a recorded copy.
//! That catches a typo, not a wrong trust anchor. A key that is well formed and
//! different from the release key passes that comparison, and the failure only
//! shows up on a user's machine, as "The signature verification failed" on every
//! update. Here the committed fixture is really signed and really verified, so a
//! mismatched pair fails on the pull request instead.
//!
//! The signature is made by the maintainer with the release private key, which
//! never enters the repository; `the_release_key_signs_what_the_embedded_pubkey_verifies`
//! fails with the exact command to produce it until `updater-signed.txt.sig`
//! exists.
//!
//! The verifier itself is proved by `throwaway-*` fixtures: a disposable key
//! pair generated with `npx tauri signer generate` and its own content, so the
//! accept and reject cases can be exercised without the release key.
//!
//! The signed content files carry no line terminator, on purpose: this
//! repository checks out with `core.autocrlf`, which rewrites line endings, and
//! a signature covers exact bytes. A file with no line ending has nothing to
//! convert, so it verifies on every platform and after every fresh clone.

use base64::Engine;
use minisign_verify::{PublicKey, Signature};

/// The fixture the release key signs; its `.sig` is what the maintainer commits.
const SIGNED_FIXTURE: &str = "tests/fixtures/updater-signed.txt";

/// The thrown-away key pair's public half, its content and that content's
/// signature. The private half was generated in a scratch directory for this
/// test data only and is never committed: it is not, and never becomes, a trust
/// anchor of this app.
const THROWAWAY_PUBKEY: &str = "tests/fixtures/throwaway-pubkey.txt";
const THROWAWAY_CONTENT: &str = "tests/fixtures/throwaway-content.txt";
const THROWAWAY_SIGNATURE: &str = "tests/fixtures/throwaway-content.txt.sig";

/// What the maintainer has to run to produce the missing release signature.
/// `-p` is deliberately a placeholder: the password lives in the maintainer's
/// own prompt and in the release CI secret, never in this repository.
const SIGN_INSTRUCTIONS: &str = "\
the release signature is missing: apps/desktop/src-tauri/tests/fixtures/updater-signed.txt.sig
  sign the fixture with the release private key, then commit only the .sig:
    cd apps/desktop
    npx tauri signer sign -f <path-to-private-key> -p <your private key password> src-tauri/tests/fixtures/updater-signed.txt
  the signer writes src-tauri/tests/fixtures/updater-signed.txt.sig next to that file; keep the key and its password out of the repository";

#[test]
fn the_verifier_accepts_the_throwaway_signature() {
    let pubkey = read_fixture(THROWAWAY_PUBKEY);
    let signature = read_fixture(THROWAWAY_SIGNATURE);
    let content = read_bytes(THROWAWAY_CONTENT);

    verify(&pubkey, &signature, &content).unwrap_or_else(|e| {
        panic!("the verifier must accept a signature made by the matching throwaway key: {e}")
    });
}

#[test]
fn the_verifier_rejects_one_changed_byte_of_content() {
    let pubkey = read_fixture(THROWAWAY_PUBKEY);
    let signature = read_fixture(THROWAWAY_SIGNATURE);
    let content = read_bytes(THROWAWAY_CONTENT);

    let mut tampered = content.clone();
    tampered[0] ^= 1;

    assert!(
        verify(&pubkey, &signature, &tampered).is_err(),
        "a signature must not verify content that changed by one byte, \
         or the update check proves nothing"
    );
}

#[test]
fn a_signature_from_another_key_does_not_verify_against_the_embedded_key() {
    let embedded = embedded_pubkey(include_str!("../tauri.conf.json"));
    let signature = read_fixture(THROWAWAY_SIGNATURE);
    let content = read_bytes(THROWAWAY_CONTENT);

    assert!(
        verify(embedded, &signature, &content).is_err(),
        "a signature from a different key must not verify against the embedded \
         updater key, or any key would be a trust anchor"
    );
}

#[test]
fn the_release_key_signs_what_the_embedded_pubkey_verifies() {
    let embedded = embedded_pubkey(include_str!("../tauri.conf.json"));
    let content = read_bytes(SIGNED_FIXTURE);
    let signature = read_fixture(&format!("{SIGNED_FIXTURE}.sig"));

    verify(embedded, &signature, &content).unwrap_or_else(|e| {
        panic!(
            "the embedded updater key does not verify the committed fixture signature, \
             so the embedded key and the release key are not the same pair and every \
             update would fail on every user's machine: {e}"
        )
    });
}

/// The minisign check `tauri-plugin-updater` 2.14 performs
/// (`verify_signature`/`base64_to_string` in its `updater.rs`): both values are
/// base64 of whole minisign text files, decoded to text, parsed, then verified.
/// `allow_legacy` is the one deviation from that source (it passes `true`): the
/// signature the release exercises carries the modern `ED` algorithm, so
/// refusing minisign's legacy `Ed` form only tightens the guard.
fn verify(pubkey_b64: &str, signature_b64: &str, content: &[u8]) -> Result<(), String> {
    let public_key_text = base64_to_string(pubkey_b64)?;
    let public_key = PublicKey::decode(&public_key_text)
        .map_err(|e| format!("the updater public key is not a minisign public key: {e}"))?;

    let signature_text = base64_to_string(signature_b64)?;
    let signature = Signature::decode(&signature_text)
        .map_err(|e| format!("the signature is not a minisign signature: {e}"))?;

    public_key
        .verify(content, &signature, false)
        .map_err(|e| format!("the signature does not verify against that public key: {e}"))
}

/// Base64 of a minisign text file, as the updater stores both the key and the
/// signature.
fn base64_to_string(base64_string: &str) -> Result<String, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_string)
        .map_err(|e| format!("{base64_string:?} is not base64: {e}"))?;
    String::from_utf8(bytes)
        .map_err(|e| format!("{base64_string:?} is not the base64 of UTF-8 text: {e}"))
}

/// The `"pubkey"` string value of the updater block: the key, then the `:` and
/// the quoted value after it. Plain string handling, as in `updater_key.rs`, so
/// no JSON crate is needed to read one string out of one file.
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

/// A committed fixture, without the newline the file ends with: the updater
/// reads the key out of JSON and the signature out of the manifest, so neither
/// carries one, and base64 decoding rejects it.
fn read_fixture(relative: &str) -> String {
    let path = fixture_path(relative);
    match std::fs::read_to_string(&path) {
        Ok(text) => text.trim_end_matches(['\r', '\n']).to_string(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            panic!("{}\n\n(reading {}: {e})", SIGN_INSTRUCTIONS, path.display())
        }
        Err(e) => panic!("{} is unreadable: {e}", path.display()),
    }
}

/// A committed fixture's exact bytes. Content is never trimmed or normalized:
/// the signature covers the file as committed, byte for byte.
fn read_bytes(relative: &str) -> Vec<u8> {
    let path = fixture_path(relative);
    match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            panic!("{}\n\n(reading {}: {e})", SIGN_INSTRUCTIONS, path.display())
        }
        Err(e) => panic!("{} is unreadable: {e}", path.display()),
    }
}

fn fixture_path(relative: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}
