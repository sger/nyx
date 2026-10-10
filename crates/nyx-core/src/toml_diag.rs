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
fn backticked_after<'a>(message: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = message.strip_prefix(prefix)?.strip_prefix('`')?;
    rest.split('`').next()
}
