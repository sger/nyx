//! Friendly hints for TOML deserialization errors.
//!
//! `toml` and `serde` produce messages such as "missing field `title`" that say
//! what went wrong but not how to fix it. This module turns the common cases
//! into a short `help` line that is shown next to the error.

/// Returns a fix-it hint for a TOML deserialization error message, if one applies.
///
/// `message` is the error text from [`toml::de::Error::message`]. `label` names
/// the file in the hint (for example `nyx.toml`), and `known_fields` lists the
/// valid keys, which are suggested when the message reports an unknown field.
///
/// Recognized messages:
///
/// - "missing field `x`": suggests adding `x = "..."` to the file.
/// - "unknown field `x`, ...": lists `known_fields` so a typo can be spotted.
/// - "... expected a TOML datetime ...": explains that dates must not be quoted.
///
/// Returns `None` for any other message.
pub(crate) fn help_for(message: &str, label: &str, known_fields: &[&str]) -> Option<String> {
    if let Some(field) = backticked_after(message, "missing field ") {
        Some(format!("add `{field} = \"...\"` to {label}"))
    } else if message.starts_with("unknown field ") {
        Some(format!(
            "check for typos; valid fields are: {}",
            known_fields.join(", ")
        ))
    } else if message.contains("expected a TOML datetime") {
        Some("write dates without quotes, e.g. `date = 2026-10-10`".to_owned())
    } else {
        None
    }
}

/// Returns the backticked word directly after `prefix`, e.g. `title` in "missing field `title`".
///
/// Returns `None` if `message` does not start with `prefix` followed by a backtick.
/// A missing closing backtick is tolerated: the rest of the message is returned.
fn backticked_after<'a>(message: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = message.strip_prefix(prefix)?.strip_prefix('`')?;
    rest.split('`').next()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIELDS: &[&str] = &["title", "base_url", "output_dir"];

    fn help(message: &str) -> Option<String> {
        help_for(message, "nyx.toml", FIELDS)
    }

    #[test]
    fn missing_field_suggests_adding_it() {
        assert_eq!(
            help("missing field `title`").as_deref(),
            Some("add `title = \"...\"` to nyx.toml")
        );
    }

    #[test]
    fn missing_field_uses_the_given_label() {
        assert_eq!(
            help_for("missing field `date`", "posts/hello.md", FIELDS).as_deref(),
            Some("add `date = \"...\"` to posts/hello.md")
        );
    }

    #[test]
    fn unknown_field_lists_known_fields() {
        assert_eq!(
            help("unknown field `titel`, expected one of `title`, `base_url`").as_deref(),
            Some("check for typos; valid fields are: title, base_url, output_dir")
        );
    }

    #[test]
    fn unknown_field_with_no_known_fields() {
        assert_eq!(
            help_for("unknown field `x`", "nyx.toml", &[]).as_deref(),
            Some("check for typos; valid fields are: ")
        );
    }

    #[test]
    fn quoted_date_explains_datetime_syntax() {
        assert_eq!(
            help("invalid type: string \"2026-10-10\", expected a TOML datetime").as_deref(),
            Some("write dates without quotes, e.g. `date = 2026-10-10`")
        );
    }

    #[test]
    fn unrelated_message_has_no_help() {
        assert_eq!(help("invalid type: integer `42`, expected a string"), None);
        assert_eq!(help(""), None);
    }

    #[test]
    fn missing_field_must_be_at_the_start() {
        assert_eq!(help("error: missing field `title`"), None);
    }

    #[test]
    fn backticked_after_extracts_the_word() {
        assert_eq!(
            backticked_after("missing field `title`", "missing field "),
            Some("title")
        );
    }

    #[test]
    fn backticked_after_stops_at_the_first_closing_backtick() {
        assert_eq!(
            backticked_after("unknown field `titel`, expected `title`", "unknown field "),
            Some("titel")
        );
    }

    #[test]
    fn backticked_after_tolerates_a_missing_closing_backtick() {
        assert_eq!(
            backticked_after("missing field `title", "missing field "),
            Some("title")
        );
    }

    #[test]
    fn backticked_after_requires_prefix_and_backtick() {
        assert_eq!(
            backticked_after("missing field title", "missing field "),
            None
        );
        assert_eq!(backticked_after("other `title`", "missing field "), None);
    }
}
