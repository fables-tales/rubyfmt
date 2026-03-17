use std::borrow::Cow;

pub fn single_to_double_quoted<'src>(
    content: &'src [u8],
    start_delim: &'src [u8],
    end_delim: &'src [u8],
) -> Cow<'src, [u8]> {
    if start_delim == b"'" || start_delim.starts_with(b"%q") {
        escape_string(
            content,
            *start_delim.last().unwrap(),
            *end_delim.last().unwrap(),
        )
        .into()
    } else {
        let start_delim_byte = *start_delim.last().unwrap();
        let end_delim_byte = *end_delim.last().unwrap();
        convert_percent_literal_escapes(content, start_delim_byte, end_delim_byte)
    }
}

/// Converts escape sequences in a percent-literal string's content so they are
/// valid inside a double-quoted string.
fn convert_percent_literal_escapes(
    content: &[u8],
    start_delim: u8,
    end_delim: u8,
) -> Cow<'_, [u8]> {
    let first_change_pos = {
        let mut pos = None;
        let mut bytes = content.iter().copied().enumerate().peekable();
        while let Some((i, c)) = bytes.next() {
            if c == b'"' {
                pos = Some(i);
                break;
            } else if c == b'\\'
                && let Some(&(_, next)) = bytes.peek()
            {
                if next == start_delim || next == end_delim {
                    pos = Some(i);
                    break;
                }
                bytes.next(); // skip next byte — treat \X as a unit
            }
        }
        pos
    };

    let Some(start) = first_change_pos else {
        return Cow::Borrowed(content); // No changes
    };

    let mut output = content[..start].to_vec();
    let mut bytes = content[start..].iter().copied().peekable();

    while let Some(c) = bytes.next() {
        if c == b'\\' {
            if let Some(&next) = bytes.peek() {
                if next == start_delim || next == end_delim {
                    // Drop the delimiter escape: \( → (
                    output.push(next);
                    bytes.next();
                } else {
                    // Write back the original with no changes
                    output.push(b'\\');
                    output.push(next);
                    bytes.next();
                }
            } else {
                output.push(b'\\');
            }
        } else if c == b'"' {
            output.extend_from_slice(b"\\\"");
        } else {
            output.push(c);
        }
    }

    Cow::Owned(output)
}

/// Escapes content for word arrays when converting to bracket delimiters.
/// This handles unescaping the original delimiter and escaping [ and ] (since they're the new delimiters)
pub fn escape_word_array_content(
    content: &[u8],
    orig_open_delim: u8,
    orig_close_delim: u8,
) -> Vec<u8> {
    // Output is always [] delimited
    const TARGET_OPEN: u8 = b'[';
    const TARGET_CLOSE: u8 = b']';

    let mut bytes = content.iter().copied().peekable();
    let mut output = Vec::new();

    while let Some(c) = bytes.next() {
        if c == b'\\' {
            if let Some(&next) = bytes.peek() {
                if next == b'\\' {
                    // Escaped backslash, keep both
                    output.push(b'\\');
                    output.push(b'\\');
                } else if next == orig_open_delim || next == orig_close_delim {
                    // Original delimiter was escaped - unescape unless it's also a target delimiter
                    if next == TARGET_OPEN || next == TARGET_CLOSE {
                        output.push(b'\\');
                    }
                    output.push(next);
                } else if next == TARGET_OPEN || next == TARGET_CLOSE {
                    // Already escaped target delimiter, keep it
                    output.push(b'\\');
                    output.push(next);
                } else {
                    output.push(b'\\');
                    output.push(next);
                }
                bytes.next();
            }
        } else if c == TARGET_OPEN || c == TARGET_CLOSE {
            // Unescaped target delimiter needs escaping
            output.push(b'\\');
            output.push(c);
        } else {
            output.push(c);
        }
    }

    output
}

fn escape_string(content: &[u8], opening_delim: u8, closing_delim: u8) -> Vec<u8> {
    if opening_delim == b'"' {
        return content.to_vec();
    }

    let mut bytes = content.iter().copied().peekable();
    let mut output = Vec::new();

    while let Some(c) = bytes.next() {
        match c {
            b'"' => {
                output.push(b'\\');
            }
            b'\\' => {
                if let Some(&next_char) = bytes.peek() {
                    match next_char {
                        b'\'' => {
                            // String#inspect strips the leading backslash from \', despite it being a valid
                            // escape character in double-quoted strings as well. Leaving the behavior the same
                            // for consistency with the previous behavior.
                            output.push(b'\'');
                            bytes.next();
                            continue;
                        }
                        // '\\' is considered an escape sequence in both single and double quoted strings
                        // and thus we don't need to "double-escape" it to "\\\\"
                        b'\\' => {
                            output.extend_from_slice(b"\\\\");
                            bytes.next();
                            continue;
                        }
                        // '\"' is a slash char and a double-quote char, not an escaped double-quote,
                        // so here we print an escaped slash character and then an escaped quote character: `"\\\""`
                        b'"' => {
                            output.extend_from_slice(b"\\\\\\\"");
                            bytes.next();
                            continue;
                        }
                        delim_byte
                            if (delim_byte == opening_delim || delim_byte == closing_delim)
                                // We only care about the "non-standard" delimiters like %() etc.
                                // For single-quoted strings, these characters should *not* be treated as escape sequences.
                                && opening_delim != b'\'' =>
                        {
                            // In percent-strings, the opening and closing delimiters can be escaped to prevent terminating the string.
                            output.push(delim_byte);
                            bytes.next();
                            continue;
                        }
                        // For everything else, this is not an escape sequence, so we need to
                        // escape the slash and then print the next character.
                        _ => {
                            output.push(b'\\');
                            output.push(b'\\');
                            output.push(next_char);
                            bytes.next();
                            continue;
                        }
                    }
                }
            }
            b'#' => {
                if let Some(&next_char) = bytes.peek() {
                    match next_char {
                        // These are shorthands for embedded variables when double-quoted,
                        // e.g. "#@foo", "#$GLOBAL", and "#{1}", so they must be escaped
                        b'$' | b'@' | b'{' => {
                            output.push(b'\\');
                            output.push(c);
                            output.push(next_char);
                            bytes.next();
                            continue;
                        }
                        _ => {}
                    };
                }
            }
            _ => {}
        };
        output.push(c);
    }

    output
}
