//! Site configuration, read from `nyx.toml` in the site root.
//!
//! A minimal config needs only a title and a base URL:
//!
//! ```toml
//! title = "My Notes"
//! base_url = "https://example.com"
//! ```
//!
//! The file is optional. Without it the site builds with [`SiteConfig::default`].

use std::{
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::toml_diag;
use crate::{NyxError, SourceFile};

/// Name of the config file, relative to the site root.
pub const CONFIG_FILE: &str = "nyx.toml";

/// Every key `SiteConfig` accepts, listed in the hint for an unknown field.
/// Must match the struct's fields; the `known_fields_match_the_struct` test checks this.
const KNOWN_FIELDS: &[&str] = &["title", "base_url", "description", "author", "output_dir"];

/// Settings for a whole site, as written in `nyx.toml`.
///
/// Unknown keys are rejected, so a misspelled key is reported instead of
/// silently ignored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteConfig {
    /// Site name, shown in page titles and feeds. Required.
    pub title: String,
    /// URL the site is served from, such as `https://example.com`. Required.
    pub base_url: String,
    /// Short summary of the site.
    #[serde(default)]
    pub description: Option<String>,
    /// Default author for pages that don't name one.
    #[serde(default)]
    pub author: Option<String>,
    /// Where the built site is written, relative to the site root. Defaults to `public`.
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
}

/// The config used when the site has no `nyx.toml`: title "Nyx site",
/// base URL `/`, and output in `public`.
impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "Nyx site".to_owned(),
            base_url: "/".to_owned(),
            description: None,
            author: None,
            output_dir: default_output_dir(),
        }
    }
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("public")
}

/// Reads and parses `nyx.toml` from the site at `root`.
///
/// A missing file is not an error: it returns [`SiteConfig::default`].
///
/// # Errors
///
/// - [`NyxError::Io`] if the file exists but can't be read.
/// - [`NyxError::Config`] if it isn't a valid config; see [`parse_config`].
pub fn load_config(root: &Path) -> Result<SiteConfig, NyxError> {
    let path = root.join(CONFIG_FILE);
    match std::fs::read_to_string(&path) {
        Ok(text) => parse_config(SourceFile::new(CONFIG_FILE, text)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(SiteConfig::default()),
        Err(source) => Err(NyxError::Io { path, source }),
    }
}

/// Parses the contents of a config file.
///
/// # Errors
///
/// Returns [`NyxError::Config`] if the text is not valid TOML or doesn't match
/// [`SiteConfig`]. The error keeps `file` so the problem can be shown in
/// context, and has a span pointing at the bad key or value when `toml`
/// reports one, plus a hint for common mistakes.
///
/// # Examples
///
/// ```
/// use nyx_core::{SourceFile, config::{CONFIG_FILE, parse_config}};
///
/// let file = SourceFile::new(CONFIG_FILE, "title = \"Notes\"\nbase_url = \"/\"\n");
/// let config = parse_config(file).unwrap();
/// assert_eq!(config.title, "Notes");
/// assert_eq!(config.output_dir, std::path::Path::new("public"));
/// ```
pub fn parse_config(file: SourceFile) -> Result<SiteConfig, NyxError> {
    toml::from_str(&file.text).map_err(|err| {
        let message = err.message().to_owned();
        NyxError::Config {
            span: err.span().map(Into::into),
            help: toml_diag::help_for(&message, CONFIG_FILE, KNOWN_FIELDS),
            message,
            file,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Span;

    fn parse(text: &str) -> Result<SiteConfig, NyxError> {
        parse_config(SourceFile::new(CONFIG_FILE, text))
    }

    fn config_error(text: &str) -> (Option<Span>, String, Option<String>) {
        match parse(text) {
            Err(NyxError::Config {
                span,
                message,
                help,
                ..
            }) => (span, message, help),
            other => panic!("expected config error, got {other:?}"),
        }
    }

    #[test]
    fn parses_all_fields() {
        let config = parse(
            r#"
title = "Spiros' Notes"
base_url = "https://example.com"
description = "Rust, iOS and tooling"
author = "Spiros"
output_dir = "dist"
"#,
        )
        .unwrap();

        assert_eq!(
            config,
            SiteConfig {
                title: "Spiros' Notes".to_owned(),
                base_url: "https://example.com".to_owned(),
                description: Some("Rust, iOS and tooling".to_owned()),
                author: Some("Spiros".to_owned()),
                output_dir: PathBuf::from("dist"),
            }
        );
    }

    #[test]
    fn optional_fields_default() {
        let config = parse("title = \"T\"\nbase_url = \"https://example.com\"\n").unwrap();
        assert_eq!(config.description, None);
        assert_eq!(config.author, None);
        assert_eq!(config.output_dir, PathBuf::from("public"));
    }

    #[test]
    fn missing_required_field_is_an_error_with_help() {
        let (_, message, help) = config_error("base_url = \"https://example.com\"\n");
        assert!(message.contains("missing field `title`"), "{message}");
        assert_eq!(help.as_deref(), Some("add `title = \"...\"` to nyx.toml"));
    }

    #[test]
    fn unknown_field_points_at_the_key() {
        let text = "title = \"T\"\nbase_url = \"https://example.com\"\ntitel = \"oops\"\n";
        let (span, message, help) = config_error(text);
        assert!(message.contains("unknown field `titel`"), "{message}");
        let span = span.unwrap();
        assert_eq!(&text[span.start..span.end], "titel");
        assert!(help.unwrap().contains("title, base_url"));
    }

    #[test]
    fn wrong_type_points_at_the_value() {
        let text = "title = 42\nbase_url = \"https://example.com\"\n";
        let (span, _, _) = config_error(text);
        let span = span.unwrap();
        assert_eq!(&text[span.start..span.end], "42");
    }

    #[test]
    fn default_values() {
        let config = SiteConfig::default();
        assert_eq!(config.title, "Nyx site");
        assert_eq!(config.base_url, "/");
        assert_eq!(config.description, None);
        assert_eq!(config.author, None);
        assert_eq!(config.output_dir, PathBuf::from("public"));
    }

    #[test]
    fn known_fields_match_the_struct() {
        let config = SiteConfig {
            description: Some(String::new()),
            author: Some(String::new()),
            ..SiteConfig::default()
        };
        let table = toml::Table::try_from(config).unwrap();
        let mut keys: Vec<&str> = table.keys().map(String::as_str).collect();
        let mut known = KNOWN_FIELDS.to_vec();
        keys.sort_unstable();
        known.sort_unstable();
        assert_eq!(keys, known);
    }

    #[test]
    fn empty_file_requires_title() {
        let (_, message, help) = config_error("");
        assert!(message.contains("missing field `title`"), "{message}");
        assert_eq!(help.as_deref(), Some("add `title = \"...\"` to nyx.toml"));
    }

    #[test]
    fn missing_base_url_is_an_error_with_help() {
        let (_, message, help) = config_error("title = \"T\"\n");
        assert!(message.contains("missing field `base_url`"), "{message}");
        assert_eq!(
            help.as_deref(),
            Some("add `base_url = \"...\"` to nyx.toml")
        );
    }

    #[test]
    fn wrong_type_has_no_help() {
        let (_, _, help) = config_error("title = 42\nbase_url = \"https://example.com\"\n");
        assert_eq!(help, None);
    }

    #[test]
    fn invalid_toml_syntax_has_a_span() {
        let text = "title = \"T\"\nbase_url = \"unterminated\n";
        let (span, _, help) = config_error(text);
        let span = span.unwrap();
        assert!(span.start >= text.find("base_url").unwrap(), "{span:?}");
        assert_eq!(help, None);
    }

    #[test]
    fn error_keeps_the_source_file() {
        let text = "title = \n";
        match parse(text) {
            Err(NyxError::Config { file, .. }) => {
                assert_eq!(file.path, Path::new(CONFIG_FILE));
                assert_eq!(&*file.text, text);
            }
            other => panic!("expected config error, got {other:?}"),
        }
    }

    #[test]
    fn unreadable_file_is_an_io_error() {
        let dir = tempfile::tempdir().unwrap();
        // A directory where the file should be can't be read as text.
        std::fs::create_dir(dir.path().join(CONFIG_FILE)).unwrap();
        match load_config(dir.path()) {
            Err(NyxError::Io { path, .. }) => assert_eq!(path, dir.path().join(CONFIG_FILE)),
            other => panic!("expected I/O error, got {other:?}"),
        }
    }

    #[test]
    fn missing_file_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load_config(dir.path()).unwrap(), SiteConfig::default());
    }

    #[test]
    fn loads_file_from_root() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(CONFIG_FILE),
            "title = \"T\"\nbase_url = \"https://example.com\"\n",
        )
        .unwrap();
        assert_eq!(load_config(dir.path()).unwrap().title, "T");
    }

    #[test]
    fn invalid_file_reports_root_relative_path() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CONFIG_FILE), "title = \n").unwrap();
        match load_config(dir.path()) {
            Err(NyxError::Config { file, .. }) => assert_eq!(file.path, Path::new(CONFIG_FILE)),
            other => panic!("expected config error, got {other:?}"),
        }
    }
}
