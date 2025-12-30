//! Base64 encoding/decoding utilities for cursor serialization.

/// Simple base64 encoding (URL-safe, no padding).
///
/// # Why Custom Implementation?
///
/// This crate avoids external dependencies for base64 encoding/decoding to:
/// 1. Minimize binary size in WASM targets
/// 2. Avoid dependency version conflicts
/// 3. Keep the implementation simple and auditable
///
/// The implementation uses URL-safe alphabet (`-_` instead of `+/`) and omits
/// padding, making cursors safe for use in query strings without additional encoding.
pub(crate) fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = u32::from(chunk.get(1).copied().unwrap_or(0));
        let b2 = u32::from(chunk.get(2).copied().unwrap_or(0));

        let n = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[(n & 0x3F) as usize] as char);
        }
    }

    result
}

/// Simple base64 decoding (URL-safe, no padding).
///
/// Accepts both URL-safe (`-_`) and standard (`+/`) alphabet for compatibility.
/// See [`base64_encode`] for rationale on custom implementation.
pub(super) fn base64_decode(input: &str) -> Result<String, ()> {
    const DECODE: [i8; 128] = {
        let mut table = [-1i8; 128];
        let mut i = 0u8;
        while i < 26 {
            table[(b'A' + i) as usize] = i as i8;
            table[(b'a' + i) as usize] = (i + 26) as i8;
            i += 1;
        }
        let mut i = 0u8;
        while i < 10 {
            table[(b'0' + i) as usize] = (i + 52) as i8;
            i += 1;
        }
        table[b'-' as usize] = 62;
        table[b'_' as usize] = 63;
        // Also support standard base64
        table[b'+' as usize] = 62;
        table[b'/' as usize] = 63;
        table
    };

    let bytes: Vec<u8> = input.bytes().collect();
    let mut result = Vec::new();

    for chunk in bytes.chunks(4) {
        let mut n = 0u32;
        let mut valid_chars = 0;

        for (i, &b) in chunk.iter().enumerate() {
            if b as usize >= 128 {
                return Err(());
            }
            let val = DECODE[b as usize];
            if val < 0 {
                return Err(());
            }
            n |= (val as u32) << (18 - i * 6);
            valid_chars += 1;
        }

        result.push((n >> 16) as u8);
        if valid_chars > 2 {
            result.push((n >> 8) as u8);
        }
        if valid_chars > 3 {
            result.push(n as u8);
        }
    }

    String::from_utf8(result).map_err(|_| ())
}

/// Escape a string for JSON per RFC 8259.
///
/// Escapes:
/// - `"` -> `\"`
/// - `\` -> `\\`
/// - Control characters (U+0000 to U+001F) -> `\uXXXX` or named escapes
pub(super) fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\x08' => result.push_str("\\b"), // backspace
            '\x0C' => result.push_str("\\f"), // form feed
            // Other control characters (U+0000 to U+001F)
            c if c.is_control() && (c as u32) < 0x20 => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            },
            c => result.push(c),
        }
    }
    result
}

/// Unescape a JSON string per RFC 8259.
pub(super) fn unescape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('b') => result.push('\x08'),
                Some('f') => result.push('\x0C'),
                Some('u') => {
                    // Parse \uXXXX escape
                    let mut hex = String::with_capacity(4);
                    for _ in 0..4 {
                        if let Some(h) = chars.next() {
                            hex.push(h);
                        }
                    }
                    if let Ok(code) = u32::from_str_radix(&hex, 16)
                        && let Some(ch) = char::from_u32(code)
                    {
                        result.push(ch);
                    }
                },
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                },
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Split JSON object into key:value pairs, respecting nesting.
pub(super) fn split_json_pairs(s: &str) -> Vec<&str> {
    let mut pairs = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;

    for (i, c) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }

        match c {
            '\\' if in_string => escape = true,
            '"' => in_string = !in_string,
            '{' | '[' if !in_string => depth += 1,
            '}' | ']' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                pairs.push(&s[start..i]);
                start = i + 1;
            },
            _ => {},
        }
    }

    if start < s.len() {
        pairs.push(&s[start..]);
    }

    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let original = "{\"id\":100,\"name\":\"test\"}";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }
}
