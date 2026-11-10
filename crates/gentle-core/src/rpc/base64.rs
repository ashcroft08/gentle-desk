//! Standard base64 (`RFC 4648`: `A-Za-z0-9+/`, `=` padding).
//!
//! pi's `images` field carries a whole file this way, and this is the only
//! place the workspace encodes it: a dependency would be a dependency for
//! twenty lines of a protocol this crate already owns.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// `bytes` as standard base64, padded to a multiple of four.
pub fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = u32::from(chunk[0]);
        let second = chunk.get(1).copied().map(u32::from);
        let third = chunk.get(2).copied().map(u32::from);
        let triple = (first << 16) | (second.unwrap_or(0) << 8) | third.unwrap_or(0);
        out.push(char::from(ALPHABET[(triple >> 18) as usize & 0x3f]));
        out.push(char::from(ALPHABET[(triple >> 12) as usize & 0x3f]));
        out.push(match second {
            Some(_) => char::from(ALPHABET[(triple >> 6) as usize & 0x3f]),
            None => '=',
        });
        out.push(match third {
            Some(_) => char::from(ALPHABET[triple as usize & 0x3f]),
            None => '=',
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rfc_vectors_encode_as_the_rfc_says() {
        // The RFC 4648 section 10 vectors, the three padding cases included.
        let cases: [(&[u8], &str); 7] = [
            (b"", ""),
            (b"f", "Zg=="),
            (b"fo", "Zm8="),
            (b"foo", "Zm9v"),
            (b"foob", "Zm9vYg=="),
            (b"fooba", "Zm9vYmE="),
            (b"foobar", "Zm9vYmFy"),
        ];
        for (bytes, expected) in cases {
            assert_eq!(
                encode(bytes),
                expected,
                "for {:?}",
                String::from_utf8_lossy(bytes)
            );
        }
    }

    #[test]
    fn both_of_the_last_places_use_the_full_alphabet() {
        // Bytes that land on `+` and `/`, the two characters an encoder that
        // masks the wrong way gets wrong.
        assert_eq!(encode(&[0xfb, 0xff]), "+/8=");
    }
}
