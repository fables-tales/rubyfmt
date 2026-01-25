use std::borrow::Cow;

use fancy_regex::Regex;

pub fn single_to_double_quoted<'src>(
    content: &'src str,
    start_delim: &'src str,
    end_delim: &'src str,
) -> Cow<'src, str> {
    if start_delim == "'" || start_delim.starts_with("%q") {
        escape_string(
            content,
            start_delim.chars().last().unwrap(),
            end_delim.chars().last().unwrap(),
        )
        .into()
    } else {
        // For percent literals, we only care about the delimiter
        // e.g. for `%<` we're looking for the `<`
        let start_delim = if let Some(stripped) = start_delim.strip_prefix('%') {
            stripped
        } else {
            start_delim
        };

        let regexp = Regex::new(&format!(
            r#"(?<!\\)(\\\\)*(\"|\\{}|\\{})"#,
            fancy_regex::escape(start_delim),
            fancy_regex::escape(end_delim)
        ))
        .unwrap();

        regexp.replace_all(content, |captures: &fancy_regex::Captures| {
            // first capture is the entire match
            let val = captures.get(0).unwrap();
            let val_str = val.as_str();
            if val_str.ends_with("\"") {
                // Ends with a quote, which we transform to `\"`
                format!("{}\\\"", &val_str[0..(val_str.len() - 1)])
            } else {
                // drop unnecessary escape
                format!(
                    "{}{}",
                    &val_str[0..(val_str.len() - 2)],
                    val_str.chars().last().unwrap()
                )
            }
        })
    }
}

/// Escapes content for word arrays when converting to bracket delimiters.
/// This handles unescaping the original delimiter and escaping [ and ] (since they're the new delimiters)
pub fn escape_word_array_content(
    content: &str,
    orig_open_delim: char,
    orig_close_delim: char,
) -> String {
    // Output is always [] delimited
    const TARGET_OPEN: char = '[';
    const TARGET_CLOSE: char = ']';

    let mut chars = content.chars().peekable();
    let mut output = String::new();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                if next == '\\' {
                    // Escaped backslash, keep both
                    output.push('\\');
                    output.push('\\');
                } else if next == orig_open_delim || next == orig_close_delim {
                    // Original delimiter was escaped - unescape unless it's also a target delimiter
                    if next == TARGET_OPEN || next == TARGET_CLOSE {
                        output.push('\\');
                    }
                    output.push(next);
                } else if next == TARGET_OPEN || next == TARGET_CLOSE {
                    // Already escaped target delimiter, keep it
                    output.push('\\');
                    output.push(next);
                } else {
                    output.push('\\');
                    output.push(next);
                }
                chars.next();
            }
        } else if c == TARGET_OPEN || c == TARGET_CLOSE {
            // Unescaped target delimiter needs escaping
            output.push('\\');
            output.push(c);
        } else {
            output.push(c);
        }
    }

    output
}

fn escape_string(content: &str, opening_delim: char, closing_delim: char) -> String {
    if opening_delim == '"' {
        return content.to_string();
    }

    let mut chars = content.chars().peekable();
    let mut output = String::new();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                output.push('\\');
            }
            '\\' => {
                if let Some(&next_char) = chars.peek() {
                    match next_char {
                        '\'' => {
                            // String#inspect strips the leading backslash from \', despite it being a valid
                            // escape character in double-quoted strings as well. Leaving the behavior the same
                            // for consistency with the previous behavior.
                            output.push('\'');
                            chars.next();
                            continue;
                        }
                        // '\\' is considered an escape sequence in both single and double quoted strings
                        // and thus we don't need to "double-escape" it to "\\\\"
                        '\\' => {
                            output.push_str("\\\\");
                            chars.next();
                            continue;
                        }
                        // '\"' is a slash char and a double-quote char, not an escaped double-quote,
                        // so here we print an escaped slash character and then an escaped quote character: `"\\\""`
                        '"' => {
                            output.push_str("\\\\\\\"");
                            chars.next();
                            continue;
                        }
                        delim_char
                            if (delim_char == opening_delim || delim_char == closing_delim)
                                // We only care about the "non-standard" delimiters like %() etc.
                                // For single-quoted strings, these characters should *not* be treated as escape sequences.
                                && opening_delim != '\'' =>
                        {
                            // In percent-strings, the opening and closing delimiters can be escaped to prevent terminating the string.
                            output.push(delim_char);
                            chars.next();
                            continue;
                        }
                        // For everything else, this is not an escape sequnce, so we need to
                        // escape the slash and then print the next character.
                        _ => {
                            output.push('\\');
                            output.push('\\');
                            output.push(next_char);
                            chars.next();
                            continue;
                        }
                    }
                }
            }
            '#' => {
                if let Some(&next_char) = chars.peek() {
                    match next_char {
                        // These are shorthands for embedded variables when double-quoted,
                        // e.g. "#@foo", "#$GLOBAL", and "#{1}", so they must be escaped
                        '$' | '@' | '{' => {
                            output.push('\\');
                            output.push(c);
                            output.push(next_char);
                            chars.next();
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
